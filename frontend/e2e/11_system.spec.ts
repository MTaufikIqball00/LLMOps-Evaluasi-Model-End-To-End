import { test, expect } from "@playwright/test";

test.describe("Module 11: System Health & SRE Diagnostics (/system)", () => {
  test("renders platform health, uptime, SQLite DB status, port allocations registry, and security posture", async ({ page }) => {
    await page.goto("/system");

    await expect(page.locator("h2")).toContainText("System Health & SRE Diagnostics");

    // Verify SRE telemetry metrics
    await expect(page.locator("text=Platform Status")).toBeVisible({ timeout: 10000 });
    await expect(page.locator("text=Uptime")).toBeVisible();
    await expect(page.locator("text=Database Engine")).toBeVisible();
    await expect(page.locator("text=Security Posture")).toBeVisible();

    // Verify port allocation table
    await expect(page.locator("text=Verified Dedicated Port Allocations (Conflict Free)")).toBeVisible();
    await expect(page.locator("text=8210").first()).toBeVisible();
    await expect(page.locator("text=3210").first()).toBeVisible();

    // Verify security controls
    await expect(page.locator("text=Automated Secret Redaction")).toBeVisible();
    await expect(page.locator("text=Strict CORS Origin Isolation")).toBeVisible();
    await expect(page.locator("text=Zero-Dependency Local Persistence")).toBeVisible();
    await expect(page.locator("text=Graceful Shutdown & Signal Handlers")).toBeVisible();
  });
});
