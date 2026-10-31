import { test, expect } from "@playwright/test";

test.describe("Module 01: Overview Dashboard (/)", () => {
  test("renders KPI metric cards, charts, and verifies zero browser console errors", async ({ page }) => {
    const consoleErrors: string[] = [];
    page.on("console", (msg) => {
      if (msg.type() === "error") {
        consoleErrors.push(msg.text());
      }
    });

    await page.goto("/");
    await expect(page).toHaveTitle(/LLMOps Control Center/i);

    // Verify main header
    await expect(page.locator("h2")).toContainText("Operations & Quality Overview");

    // Verify KPI metric cards
    await expect(page.locator("text=Total Requests")).toBeVisible({ timeout: 10000 });
    await expect(page.locator("text=Latency (p95)")).toBeVisible();
    await expect(page.locator("text=Token Consumption")).toBeVisible();
    await expect(page.locator("text=Monthly Spend")).toBeVisible();

    // Verify charts and distribution cards
    await expect(page.locator("text=Latency Percentiles (Rolling Window)")).toBeVisible();
    await expect(page.locator("text=Model Usage Distribution")).toBeVisible();

    // Verify status badge
    await expect(page.locator("text=All Systems Nominal")).toBeVisible();

    // Ensure no unhandled runtime errors in console
    const criticalErrors = consoleErrors.filter(
      (err) => !err.includes("favicon") && !err.includes("Download the React DevTools")
    );
    expect(criticalErrors).toHaveLength(0);
  });
});
