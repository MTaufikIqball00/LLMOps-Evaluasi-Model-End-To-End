import { test, expect } from "@playwright/test";

test.describe("Module 07: Token Usage & Costs (/costs)", () => {
  test("renders cost KPI summary, provider breakdown, pricing catalog, and edits budget settings", async ({ page }) => {
    await page.goto("/costs");

    await expect(page.locator("h2")).toContainText("Token Usage & Cost Tracking");

    // Verify KPI summary
    await expect(page.locator("text=Current Month Spend")).toBeVisible({ timeout: 10000 });
    await expect(page.locator("text=Monthly Budget").first()).toBeVisible();
    await expect(page.locator("text=Projected End-of-Month")).toBeVisible();

    // Verify tables
    await expect(page.locator("text=Spend Breakdown by Inference Provider")).toBeVisible();
    await expect(page.locator("text=Standard Token Pricing Catalog")).toBeVisible();

    // Configure budget limits toggle
    const budgetBtn = page.locator("button:has-text('Configure Budget Limits')");
    await budgetBtn.click();

    // Verify budget editor panel is visible
    await expect(page.locator("h3:has-text('Update Monthly Budget & Alert Trigger')")).toBeVisible();

    // Modify budget value and save
    const budgetInput = page.locator("input[type='number']").first();
    await budgetInput.fill("2800");

    await page.locator("button:has-text('Save Budget Limits')").click();

    // Verify update reflected on page
    await expect(page.locator("text=$2800.00").or(page.locator("text=$2,800.00")).or(page.locator("text=2800"))).toBeVisible({ timeout: 10000 });
  });
});
