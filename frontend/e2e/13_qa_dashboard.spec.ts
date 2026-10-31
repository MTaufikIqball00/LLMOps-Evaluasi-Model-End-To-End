import { test, expect } from "@playwright/test";

test.describe("Module 13: Autonomous QA Dashboard & Reporting (/qa)", () => {
  test("renders audit scorecards, multi-stage pipeline, tabs, and repair proposals", async ({ page }) => {
    await page.goto("/qa");

    await expect(page.locator("h2")).toContainText("Autonomous AI QA & Software Audit Engine");

    // Verify KPI metric cards
    await expect(page.locator("text=Pass Rate")).toBeVisible({ timeout: 10000 });
    await expect(page.locator("text=Quality Score")).toBeVisible();
    await expect(page.locator("text=Discovered Defects")).toBeVisible();
    await expect(page.locator("text=Feature Coverage").first()).toBeVisible();

    // Verify multi-stage pipeline ribbon
    await expect(page.locator("text=Multi-Stage Verification Pipeline")).toBeVisible();
    await expect(page.locator("text=1. Discovery")).toBeVisible();
    await expect(page.locator("text=2. Axum API")).toBeVisible();
    await expect(page.locator("text=4. Playwright E2E")).toBeVisible();
    await expect(page.locator("text=5. Security Audit")).toBeVisible();

    // Verify tab navigation: Feature Coverage table
    await expect(page.locator("text=Feature Forensics & Coverage Matrix")).toBeVisible();
    await expect(page.locator("text=FEAT-001")).toBeVisible();

    // Switch to Test Case Results tab
    await page.locator("button:has-text('Test Case Results')").click();
    await expect(page.locator("input[placeholder='Search test cases...']")).toBeVisible();

    // Switch to Defect Findings tab
    await page.locator("button:has-text('Defect Findings & Root Causes')").click();
    await expect(page.locator("text=SEC-VULN-001")).toBeVisible();
    await expect(page.locator("text=Root Cause Category: Backend logic bug").first()).toBeVisible();

    // Switch to AI Repair Advisor tab
    await page.locator("button:has-text('AI Repair Advisor Diffs')").click();
    await expect(page.locator("text=Deterministic AI Repair Advisor")).toBeVisible();
    await expect(page.locator("text=PATCH-001")).toBeVisible();
  });
});
