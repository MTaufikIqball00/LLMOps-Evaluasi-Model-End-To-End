use llmops_backend::persistence;
use uuid::Uuid;

#[tokio::test]
async fn test_clean_database_initialization_and_migrations() {
    let db_id = Uuid::new_v4().to_string();
    let db_dir = std::path::Path::new("target/tmp/test_dbs");
    tokio::fs::create_dir_all(&db_dir).await.ok();
    let db_path = format!("target/tmp/test_dbs/db_mig_{}.db", db_id);
    let db_url = format!("sqlite://{}?mode=rwc", db_path);

    // Initial clean bootstrap
    let pool = persistence::init_db(&db_url)
        .await
        .expect("Database initialization must succeed from clean environment");

    // Verify all 10 tables exist
    let tables: Vec<(String,)> = sqlx::query_as(
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name ASC"
    )
    .fetch_all(&pool)
    .await
    .expect("Must fetch tables list");

    let table_names: Vec<String> = tables.into_iter().map(|t| t.0).collect();
    let expected_tables = vec![
        "budget_settings",
        "cicd_workflows",
        "deployments",
        "evaluation_test_cases",
        "evaluations",
        "experiments",
        "integrations_config",
        "models",
        "pricing_catalog",
        "request_traces",
    ];

    for expected in expected_tables {
        assert!(
            table_names.contains(&expected.to_string()),
            "Table '{}' must exist in verified database schema",
            expected
        );
    }

    // Verify seed data was inserted
    let model_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM models")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(model_count.0 >= 4, "Must seed baseline models");

    // Clean up
    drop(pool);
    let _ = std::fs::remove_file(&db_path);
}

#[tokio::test]
async fn test_primary_and_unique_key_constraints() {
    let db_id = Uuid::new_v4().to_string();
    let db_path = format!("target/tmp/test_dbs/db_pk_{}.db", db_id);
    let db_url = format!("sqlite://{}?mode=rwc", db_path);

    let pool = persistence::init_db(&db_url).await.unwrap();

    // 1. Primary Key uniqueness on models
    let duplicate_insert = sqlx::query(
        r#"
        INSERT INTO models (id, name, framework, task, stage, version)
        VALUES ('m-llama31-70b', 'Duplicate Model', 'vLLM', 'Text', 'Dev', 'v1.0')
        "#
    )
    .execute(&pool)
    .await;

    assert!(
        duplicate_insert.is_err(),
        "Inserting duplicate primary key 'm-llama31-70b' must fail constraint"
    );

    // 2. Unique constraint on integrations_config.service_name
    let duplicate_service = sqlx::query(
        r#"
        INSERT INTO integrations_config (id, service_name, last_status)
        VALUES ('int-duplicate', 'mlflow', 'Connected')
        "#
    )
    .execute(&pool)
    .await;

    assert!(
        duplicate_service.is_err(),
        "Inserting duplicate service_name 'mlflow' must fail UNIQUE constraint"
    );

    // 3. NOT NULL constraint on models.name
    let null_name = sqlx::query(
        r#"
        INSERT INTO models (id, name, framework, task, stage, version)
        VALUES ('m-null-test', NULL, 'vLLM', 'Text', 'Dev', 'v1.0')
        "#
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

    // Enable foreign keys explicitly in SQLite session
    sqlx::query("PRAGMA foreign_keys = ON;").execute(&pool).await.unwrap();

    let eval_id = "eval-fk-test-01";
    sqlx::query(
        r#"
        INSERT INTO evaluations (id, name, model_name, dataset_name, evaluator_framework, status)
        VALUES (?, 'Cascade Test Eval', 'TestModel', 'Dataset', 'Ragas', 'Completed')
        "#
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
        "#
    )
    .bind(eval_id)
    .bind(eval_id)
    .execute(&pool)
    .await
    .unwrap();

    // Verify test cases exist
    let before_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM evaluation_test_cases WHERE evaluation_id = ?"
    )
    .bind(eval_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(before_count.0, 2);

    // Delete parent evaluation
    sqlx::query("DELETE FROM evaluations WHERE id = ?")
        .bind(eval_id)
        .execute(&pool)
        .await
        .unwrap();

    // Verify child test cases were deleted via ON DELETE CASCADE
    let after_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM evaluation_test_cases WHERE evaluation_id = ?"
    )
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

    // Valid insert in transaction
    sqlx::query(
        r#"
        INSERT INTO models (id, name, framework, task, stage, version)
        VALUES ('m-tx-01', 'Tx Test Model', 'vLLM', 'Text', 'Dev', 'v1.0')
        "#
    )
    .execute(&mut *tx)
    .await
    .unwrap();

    // Deliberate error in transaction (violate PK)
    let err_query = sqlx::query(
        r#"
        INSERT INTO models (id, name, framework, task, stage, version)
        VALUES ('m-tx-01', 'Tx Test Duplicate', 'vLLM', 'Text', 'Dev', 'v1.0')
        "#
    )
    .execute(&mut *tx)
    .await;
    assert!(err_query.is_err());

    // Explicit rollback
    tx.rollback().await.unwrap();

    // Verify row was NOT persisted
    let check: Option<(String,)> = sqlx::query_as("SELECT id FROM models WHERE id = 'm-tx-01'")
        .fetch_optional(&pool)
        .await
        .unwrap();
    assert!(check.is_none(), "Rolled back transaction data must not exist");

    drop(pool);
    let _ = std::fs::remove_file(&db_path);
}
