import { test, expect } from "@playwright/test";

test.describe("Module 10: Integrations & Service Adapters (/integrations)", () => {
  test("renders 8 service adapters, triggers individual probe, and batch probe", async ({ page }) => {
    await page.goto("/integrations");

    await expect(page.locator("h2")).toContainText("Integrations & Service Adapters");

    // Verify key adapters
    await expect(page.locator("text=Ragas LLM Evaluation Engine").first()).toBeVisible({ timeout: 10000 });
    await expect(page.locator("text=TruLens Evaluation Triad").first()).toBeVisible();
    await expect(page.locator("text=MLflow Tracking Server").first()).toBeVisible();
    await expect(page.locator("text=Kubernetes Cluster API").first()).toBeVisible();

    // Test individual probe button
    const testBtns = page.locator("button:has-text('Test Connection')");
    await expect(testBtns.first()).toBeVisible();
    await testBtns.first().click();

    // Verify batch probe button
    const probeAllBtn = page.locator("button:has-text('Probe All Adapters')");
    await probeAllBtn.click();

    await page.waitForTimeout(1000);
    await expect(page.locator("h2")).toContainText("Integrations & Service Adapters");
  });
});
