import { test, expect } from "@playwright/test";

test.describe("Module 12: Sidebar Navigation & System Resilience", () => {
  const routes = [
    { name: "Overview", path: "/", headerText: "Operations & Quality Overview" },
    { name: "Model Registry", path: "/models", headerText: "Model Registry & Lifecycle" },
    { name: "Experiments", path: "/experiments", headerText: "Experiment Tracking & Diffing" },
    { name: "LLM Evaluations", path: "/evaluations", headerText: "LLM Evaluation & Hallucination Analysis" },
    { name: "Production Monitoring", path: "/monitoring", headerText: "Production Telemetry & Observability" },
    { name: "Traces & Requests", path: "/traces", headerText: "Request Traces & Debugger" },
    { name: "Token Usage & Costs", path: "/costs", headerText: "Token Usage & Cost Tracking" },
    { name: "Deployments", path: "/deployments", headerText: "Kubernetes Serving Deployments" },
    { name: "CI/CD Pipelines", path: "/cicd", headerText: "CI/CD Pipeline Automation" },
    { name: "Integrations", path: "/integrations", headerText: "Integrations & Service Adapters" },
    { name: "System Health", path: "/system", headerText: "System Health & SRE Diagnostics" },
  ];

  test("seamlessly navigates across all 11 core routes via Sidebar", async ({ page }) => {
    await page.goto("/");

    for (const r of routes) {
      // Click navigation item in sidebar
      const navLink = page.locator(`aside a[href='${r.path}']`);
      await expect(navLink).toBeVisible();
      await navLink.click();

      // Verify URL and page header match expected
      await expect(page).toHaveURL(new RegExp(`${r.path.replace("/", "\\/")}$`));
      await expect(page.locator("h2")).toContainText(r.headerText, { timeout: 10000 });
    }
  });

  test("maintains responsive layout on mobile viewport", async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto("/");
    await expect(page.locator("h1")).toContainText("LLMOps Center");
    await expect(page.locator("h2")).toContainText("Operations & Quality Overview");
  });
});
