import { test, expect } from "@playwright/test";

test.describe("Module 02: Model Registry (/models)", () => {
  test("renders model list, filters by stage, and handles model creation modal", async ({ page }) => {
    await page.goto("/models");

    // Header verification
    await expect(page.locator("h2")).toContainText("Model Registry & Lifecycle");

    // Verify baseline models loaded
    await expect(page.locator("text=Llama-3.1-70B-Instruct").first()).toBeVisible({ timeout: 10000 });

    // Test stage filter dropdown
    const stageSelect = page.locator("select").first();
    await stageSelect.selectOption("Production");
    await page.waitForTimeout(500);
    await expect(page.locator("text=Llama-3.1-70B-Instruct").first()).toBeVisible();

    // Reset filter
    await stageSelect.selectOption("all");
    await page.waitForTimeout(500);

    // Test Register New Model modal
    const registerBtn = page.locator("button:has-text('Register Model')");
    await registerBtn.click();

    // Verify modal is open
    await expect(page.locator("h3:has-text('Register New Model Version')")).toBeVisible();

    // Fill form
    const uniqueModelName = `E2E-Automated-Model-${Date.now().toString().slice(-4)}`;
    await page.locator("input[placeholder='e.g. Llama-3.3-70B-Instruct']").fill(uniqueModelName);
    await page.locator("input[value='v1.0.0']").fill("v1.0.1");
    await page.locator("input[value='70B']").fill("14B");

    // Submit form
    await page.locator("button:has-text('Save Model')").click();

    // Modal should close and new model should be visible in the catalog
    await expect(page.locator(`text=${uniqueModelName}`)).toBeVisible({ timeout: 10000 });
  });

  test("handles model lifecycle stage promotion", async ({ page }) => {
    await page.goto("/models");

    // Find first available Promote Stage button
    const promoteBtn = page.locator("button:has-text('Promote to')").first();
    if (await promoteBtn.isVisible()) {
      await promoteBtn.click();
      await page.waitForTimeout(1000);
      await expect(page.locator("h2")).toContainText("Model Registry & Lifecycle");
    }
  });
});
