use llmops_backend::persistence;
use uuid::Uuid;

#[tokio::test]
async fn test_clean_database_initialization_and_migrations() {
    let db_id = Uuid::new_v4().to_string();
    let db_dir = std::path::Path::new("target/tmp/test_dbs");
    tokio::fs::create_dir_all(&db_dir).await.ok();
    let db_path = format!("target/tmp/test_dbs/db_mig_{}.db", db_id);
    let db_url = format!("sqlite://{}?mode=rwc", db_path);

    let pool = persistence::init_db(&db_url)
        .await
        .expect("Database initialization must succeed from clean environment");

    let tables: Vec<(String,)> = sqlx::query_as(
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name ASC",
    )
    .fetch_all(&pool)
    .await
    .expect("Must fetch tables list");

    let table_names: Vec<String> = tables.into_iter().map(|t| t.0).collect();
    let expected_tables = vec![
        "alert_events",
        "alert_rules",
        "budget_settings",
        "cicd_workflows",
        "deployments",
        "evaluation_test_cases",
        "evaluations",
        "experiments",
        "integrations_config",
        "models",
        "pricing_catalog",
        "prompts",
        "request_traces",
    ];

    for expected in expected_tables {
        assert!(
            table_names.contains(&expected.to_string()),
            "Table '{}' must exist in verified database schema",
            expected
        );
    }

    let model_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM models")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(model_count.0 >= 4, "Must seed baseline models");

    drop(pool);
    let _ = std::fs::remove_file(&db_path);
}

#[tokio::test]
async fn test_primary_and_unique_key_constraints() {
    let db_id = Uuid::new_v4().to_string();
    let db_path = format!("target/tmp/test_dbs/db_pk_{}.db", db_id);
    let db_url = format!("sqlite://{}?mode=rwc", db_path);

    let pool = persistence::init_db(&db_url).await.unwrap();

    let duplicate_insert = sqlx::query(
        r#"
        INSERT INTO models (id, name, framework, task, stage, version)
        VALUES ('m-llama31-70b', 'Duplicate Model', 'vLLM', 'Text', 'Dev', 'v1.0')
        "#,
    )
    .execute(&pool)
    .await;

    assert!(
        duplicate_insert.is_err(),
        "Inserting duplicate primary key 'm-llama31-70b' must fail constraint"
    );

    let duplicate_service = sqlx::query(
        r#"
        INSERT INTO integrations_config (id, service_name, last_status)
        VALUES ('int-duplicate', 'mlflow', 'Connected')
        "#,
    )
    .execute(&pool)
    .await;

    assert!(
        duplicate_service.is_err(),
        "Inserting duplicate service_name 'mlflow' must fail UNIQUE constraint"
    );

    let null_name = sqlx::query(
        r#"
        INSERT INTO models (id, name, framework, task, stage, version)
        VALUES ('m-null-test', NULL, 'vLLM', 'Text', 'Dev', 'v1.0')
        "#,
    )
    .execute(&pool)
    .await;

    assert!(
        null_name.is_err(),
        "Inserting NULL into NOT NULL column 'name' must fail"
    );

    drop(pool);
    let _ = std::fs::remove_file(&db_path);
}

#[tokio::test]
async fn test_foreign_key_cascading_deletes() {
    let db_id = Uuid::new_v4().to_string();
    let db_path = format!("target/tmp/test_dbs/db_fk_{}.db", db_id);
    let db_url = format!("sqlite://{}?mode=rwc", db_path);

    let pool = persistence::init_db(&db_url).await.unwrap();

    // Foreign keys are automatically enabled on every pooled connection via after_connect
    let eval_id = "eval-fk-test-01";
    sqlx::query(
        r#"
        INSERT INTO evaluations (id, name, model_name, dataset_name, evaluator_framework, status)
        VALUES (?, 'Cascade Test Eval', 'TestModel', 'Dataset', 'Ragas', 'Completed')
        "#,
    )
    .bind(eval_id)
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        INSERT INTO evaluation_test_cases (id, evaluation_id, question, generated_answer)
        VALUES ('tc-fk-01', ?, 'Question 1', 'Answer 1'),
               ('tc-fk-02', ?, 'Question 2', 'Answer 2')
        "#,
    )
    .bind(eval_id)
    .bind(eval_id)
    .execute(&pool)
    .await
    .unwrap();

    let before_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM evaluation_test_cases WHERE evaluation_id = ?")
            .bind(eval_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(before_count.0, 2);

    sqlx::query("DELETE FROM evaluations WHERE id = ?")
        .bind(eval_id)
        .execute(&pool)
        .await
        .unwrap();

    let after_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM evaluation_test_cases WHERE evaluation_id = ?")
            .bind(eval_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(after_count.0, 0, "Child test cases must be cascade-deleted");

    drop(pool);
    let _ = std::fs::remove_file(&db_path);
}

#[tokio::test]
async fn test_transaction_rollback_integrity() {
    let db_id = Uuid::new_v4().to_string();
    let db_path = format!("target/tmp/test_dbs/db_tx_{}.db", db_id);
    let db_url = format!("sqlite://{}?mode=rwc", db_path);

    let pool = persistence::init_db(&db_url).await.unwrap();

    let mut tx = pool.begin().await.unwrap();

    sqlx::query(
        r#"
        INSERT INTO models (id, name, framework, task, stage, version)
        VALUES ('m-tx-01', 'Tx Test Model', 'vLLM', 'Text', 'Dev', 'v1.0')
        "#,
    )
    .execute(&mut *tx)
    .await
    .unwrap();

    let err_query = sqlx::query(
        r#"
        INSERT INTO models (id, name, framework, task, stage, version)
        VALUES ('m-tx-01', 'Tx Test Duplicate', 'vLLM', 'Text', 'Dev', 'v1.0')
        "#,
    )
    .execute(&mut *tx)
    .await;
    assert!(err_query.is_err());

    tx.rollback().await.unwrap();

    let check: Option<(String,)> = sqlx::query_as("SELECT id FROM models WHERE id = 'm-tx-01'")
        .fetch_optional(&pool)
        .await
        .unwrap();
    assert!(
        check.is_none(),
        "Rolled back transaction data must not exist"
    );

    drop(pool);
    let _ = std::fs::remove_file(&db_path);
}

#[tokio::test]
async fn test_sqlite_pragmas_enabled_automatically() {
    let db_id = Uuid::new_v4().to_string();
    tokio::fs::create_dir_all("target/tmp/test_dbs").await.ok();
    let db_path = format!("target/tmp/test_dbs/db_pragma_{}.db", db_id);
    let db_url = format!("sqlite://{}?mode=rwc", db_path);

    let pool = persistence::init_db(&db_url).await.unwrap();

    let fk_enabled: (i64,) = sqlx::query_as("PRAGMA foreign_keys;")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(fk_enabled.0, 1, "SQLite foreign_keys must default to ON");

    let journal_mode: (String,) = sqlx::query_as("PRAGMA journal_mode;")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        journal_mode.0.to_ascii_lowercase(),
        "wal",
        "SQLite journal_mode must default to WAL"
    );

    drop(pool);
    let _ = std::fs::remove_file(&db_path);
    let _ = std::fs::remove_file(format!("{}-wal", db_path));
    let _ = std::fs::remove_file(format!("{}-shm", db_path));
}

#[test]
fn test_postgres_sql_adapter() {
    assert!(persistence::is_postgres_url(
        "postgres://user:pass@localhost:5432/llmops"
    ));
    assert!(persistence::is_postgres_url(
        "postgresql://user:pass@localhost/db"
    ));
    assert!(!persistence::is_postgres_url(
        "sqlite://data/llmops.db?mode=rwc"
    ));

    let raw = "INSERT INTO models (id, name, created_at) VALUES (?, 'foo?', CURRENT_TIMESTAMP)";
    let adapted = persistence::adapt_sql(raw, true);
    assert_eq!(
        adapted,
        "INSERT INTO models (id, name, created_at) VALUES ($1, 'foo?', to_char(CURRENT_TIMESTAMP AT TIME ZONE 'UTC', 'YYYY-MM-DD HH24:MI:SS'))"
    );

    let ddl = "CREATE TABLE t (a INTEGER, b REAL, created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP, updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP, checked_at TIMESTAMP);";
    let pg_ddl = persistence::adapt_migration_for_postgres(ddl);
    assert!(pg_ddl.contains("BIGINT"));
    assert!(pg_ddl.contains("DOUBLE PRECISION"));
    assert_eq!(pg_ddl.matches("TEXT NOT NULL DEFAULT to_char").count(), 2);
    assert!(pg_ddl.contains("checked_at TEXT"));
}

#[tokio::test]
async fn test_legacy_sqlite_timestamp_migration() {
    let db_id = Uuid::new_v4().to_string();
    tokio::fs::create_dir_all("target/tmp/test_dbs").await.ok();
    let db_path = format!("target/tmp/test_dbs/db_legacy_ts_{}.db", db_id);
    let db_url = format!("sqlite://{}?mode=rwc", db_path);

    sqlx::any::install_default_drivers();
    let legacy_pool = sqlx::any::AnyPoolOptions::new()
        .max_connections(1)
        .connect(&db_url)
        .await
        .unwrap();

    sqlx::query(
        r#"
        CREATE TABLE models (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            framework TEXT NOT NULL,
            task TEXT NOT NULL,
            stage TEXT NOT NULL,
            version TEXT NOT NULL,
            parameters_count TEXT,
            artifact_uri TEXT,
            description TEXT,
            tags TEXT,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(&legacy_pool)
    .await
    .unwrap();

    sqlx::query(
        r#"
        INSERT INTO models (id, name, framework, task, stage, version, created_at, updated_at)
        VALUES ('m-legacy-01', 'Legacy Timestamp Model', 'vLLM', 'Text Generation', 'Production', 'v0.1.0', '2025-01-01 00:00:00', '2025-01-01 00:00:00')
        "#,
    )
    .execute(&legacy_pool)
    .await
    .unwrap();
    drop(legacy_pool);

    let migrated_pool = persistence::init_db(&db_url)
        .await
        .expect("init_db must transparently migrate legacy TIMESTAMP tables");

    let legacy_row: llmops_backend::domain::Model =
        sqlx::query_as("SELECT * FROM models WHERE id = 'm-legacy-01'")
            .fetch_one(&migrated_pool)
            .await
            .expect("Migrated legacy TIMESTAMP row must decode cleanly via AnyPool");

    assert_eq!(legacy_row.name, "Legacy Timestamp Model");
    assert_eq!(legacy_row.created_at, "2025-01-01 00:00:00");

    let models_ddl: (String,) =
        sqlx::query_as("SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'models'")
            .fetch_one(&migrated_pool)
            .await
            .unwrap();
    assert!(!models_ddl.0.contains("TIMESTAMP NOT NULL"));
    assert!(models_ddl
        .0
        .contains("TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP"));

    drop(migrated_pool);
    let _ = std::fs::remove_file(&db_path);
    let _ = std::fs::remove_file(format!("{}-wal", db_path));
    let _ = std::fs::remove_file(format!("{}-shm", db_path));
}
