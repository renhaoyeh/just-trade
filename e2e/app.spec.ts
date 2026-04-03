import { test, expect } from "@playwright/test";

test.describe("Dashboard", () => {
  test("should display app heading", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("text=Just Trade").first()).toBeVisible();
  });

  test("should render stat cards", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("text=Total Portfolio")).toBeVisible();
    await expect(page.locator("text=Today's P&L")).toBeVisible();
    await expect(page.locator("text=Buying Power")).toBeVisible();
    await expect(page.locator("text=Open Orders")).toBeVisible();
  });

  test("should switch between tabs", async ({ page }) => {
    await page.goto("/");

    // Default tab should show Holdings
    await expect(page.locator("text=Holdings")).toBeVisible();

    // Switch to Recent Trades via the tab trigger
    await page.locator('[data-slot="tabs-trigger"]', { hasText: "Recent Trades" }).click();
    await expect(page.locator("text=Today's executed orders")).toBeVisible();

    // Switch to Watchlist via the tab trigger
    await page.locator('[data-slot="tabs-trigger"]', { hasText: "Watchlist" }).click();
    await expect(page.locator("text=META")).toBeVisible();
  });

  test("should have a sidebar with navigation", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator('[data-slot="sidebar"]')).toBeVisible();
    await expect(page.locator("text=Dashboard").first()).toBeVisible();
    await expect(page.locator("text=Portfolio").first()).toBeVisible();
    await expect(page.locator("text=Trade").first()).toBeVisible();
  });
});
