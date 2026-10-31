import { test, expect } from "@playwright/test";

test.describe("Module 05: Production Monitoring (/monitoring)", () => {
  test("renders telemetry time-series, percentiles, latency histogram, and status indicators", async ({ page }) => {
    await page.goto("/monitoring");

    await expect(page.locator("h2")).toContainText("Production Telemetry & Observability");

    // Verify timeseries charts & metrics
    await expect(page.locator("text=Inference Request Throughput (RPM)")).toBeVisible({ timeout: 10000 });
    await expect(page.locator("text=Response Latency Percentiles (p50 / p95 / p99)")).toBeVisible();

    // Verify latency histogram
    await expect(page.locator("text=Latency Distribution Histogram")).toBeVisible();
    await expect(page.locator("text=< 150 ms")).toBeVisible();
    await expect(page.locator("text=>= 500 ms")).toBeVisible();

    // Verify Grafana and Prometheus observability cards
    await expect(page.locator("text=Grafana Observability Suite")).toBeVisible();
    await expect(page.locator("text=Prometheus Port 9190 Active")).toBeVisible();
  });
});
