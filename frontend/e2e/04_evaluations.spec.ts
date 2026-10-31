import { test, expect } from "@playwright/test";

test.describe("Module 04: LLM Evaluations & Hallucination Analysis (/evaluations)", () => {
  test("renders evaluation benchmark suites, test case inspector, and triggers new evaluation", async ({ page }) => {
    await page.goto("/evaluations");

    await expect(page.locator("h2")).toContainText("LLM Evaluation & Hallucination Analysis");

    // Verify seeded evaluation suites
    await expect(page.locator("text=Q3 Enterprise Policy RAG Benchmark").first()).toBeVisible({ timeout: 10000 });

    // Verify test case verification panel
    await expect(page.locator("text=Individual Test Case Verification").first()).toBeVisible();

    // Verify metric definitions toggle
    const defsBtn = page.locator("button:has-text('Metric Definitions')");
    await expect(defsBtn).toBeVisible();
    await defsBtn.click();
    await expect(page.locator("text=Faithfulness").first()).toBeVisible();
    await expect(page.locator("text=Hallucination Score").first()).toBeVisible();

    // Trigger Evaluation modal
    const triggerBtn = page.locator("button:has-text('New Evaluation Run')");
    await triggerBtn.click();

    await expect(page.locator("h3:has-text('Trigger New Evaluation Benchmark')")).toBeVisible();

    const uniqueEvalName = `E2E-Automated-Eval-${Date.now().toString().slice(-4)}`;
    await page.locator("input[required]").first().fill(uniqueEvalName);

    // Submit evaluation
    await page.locator("button:has-text('Start Benchmark Job')").click();

    // Verify evaluation appears
    await expect(page.locator(`text=${uniqueEvalName}`).first()).toBeVisible({ timeout: 10000 });
  });
});
