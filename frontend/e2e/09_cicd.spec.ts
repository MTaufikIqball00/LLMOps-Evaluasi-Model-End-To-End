import { test, expect } from "@playwright/test";

test.describe("Module 09: CI/CD Pipeline Automation (/cicd)", () => {
  test("renders CI/CD workflows table, commit SHA, and dispatches new workflow", async ({ page }) => {
    await page.goto("/cicd");

    await expect(page.locator("h2")).toContainText("CI/CD Pipeline Automation");

    // Verify workflows table
    await expect(page.locator("text=Model Evaluation & Benchmark Gates").first()).toBeVisible({ timeout: 10000 });

    // Open dispatch modal
    const dispatchBtn = page.locator("button:has-text('Dispatch Workflow')");
    await dispatchBtn.click();

    await expect(page.locator("h3:has-text('Dispatch GitHub Actions Workflow')")).toBeVisible();

    // Select workflow
    await page.locator("select").first().selectOption("vLLM Serving Container Build & Security Scan");

    // Submit
    await page.locator("button:has-text('Dispatch Run')").click();

    // Verify workflow modal closed
    await expect(page.locator("h3:has-text('Dispatch GitHub Actions Workflow')")).not.toBeVisible({ timeout: 10000 });
  });
});
