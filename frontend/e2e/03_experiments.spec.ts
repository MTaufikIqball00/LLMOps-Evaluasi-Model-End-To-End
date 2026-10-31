import { test, expect } from "@playwright/test";

test.describe("Module 03: Experiments Tracking (/experiments)", () => {
  test("renders experiment runs and handles side-by-side run comparison", async ({ page }) => {
    await page.goto("/experiments");

    await expect(page.locator("h2")).toContainText("Experiment Tracking & Diffing");

    // Verify seeded runs
    await expect(page.locator("text=llama31-customer-support-lora-r16").first()).toBeVisible({ timeout: 10000 });

    // Verify table section exists
    await expect(page.locator("text=Registered Experiment Runs")).toBeVisible();

    // Verify comparison section exists
    await expect(page.locator("text=Side-by-Side Run Hyperparameter & Metric Comparison")).toBeVisible();
  });
});
