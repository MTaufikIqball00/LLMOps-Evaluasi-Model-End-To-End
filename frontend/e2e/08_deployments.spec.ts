import { test, expect } from "@playwright/test";

test.describe("Module 08: Kubernetes Serving Deployments (/deployments)", () => {
  test("renders workload cards, pod replicas, hardware telemetry, and simulates rolling restart", async ({ page }) => {
    await page.goto("/deployments");

    await expect(page.locator("h2")).toContainText("Kubernetes Serving Deployments");

    // Verify seeded serving workloads
    await expect(page.locator("text=llama-3.1-70b-vllm-serving").first()).toBeVisible({ timeout: 10000 });
    await expect(page.locator("text=mistral-large-candidate-serving").first()).toBeVisible();

    // Verify resource metrics displayed
    await expect(page.locator("text=Pods Ready / Desired:").first()).toBeVisible();
    await expect(page.locator("text=CPU Usage:").first()).toBeVisible();
    await expect(page.locator("text=VRAM / Memory:").first()).toBeVisible();

    // Test Rolling Restart action button
    const restartBtn = page.locator("button:has-text('Rollout Restart')").first();
    await expect(restartBtn).toBeVisible();
    await restartBtn.click();

    // Verify it updates without unhandled page error
    await page.waitForTimeout(1000);
    await expect(page.locator("text=llama-3.1-70b-vllm-serving").first()).toBeVisible();
  });
});
