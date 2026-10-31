import { test, expect } from "@playwright/test";

test.describe("Module 06: Request Traces & Debugger (/traces)", () => {
  test("renders traces waterfall, secret scrubbing banner, and detailed span inspector", async ({ page }) => {
    await page.goto("/traces");

    await expect(page.locator("h2")).toContainText("Request Traces & Debugger");

    // Verify secret scrubbing notice
    await expect(page.locator("text=Automated Secret Scrubbing Active")).toBeVisible({ timeout: 10000 });

    // Verify seeded traces in list
    const firstTrace = page.locator(".cursor-pointer").first();
    await expect(firstTrace).toBeVisible({ timeout: 10000 });
    await firstTrace.click();

    // Verify span detail viewer shows sanitized payload
    await expect(page.locator("text=Redacted Prompt Payload")).toBeVisible();

    // Test model filter
    const modelSelect = page.locator("select").first();
    await modelSelect.selectOption("Llama-3.1-70B-Instruct");
    await page.waitForTimeout(500);
    await expect(page.locator("text=rag_orchestrator::execute").first()).toBeVisible();
  });
});
