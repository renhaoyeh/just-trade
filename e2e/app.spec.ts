import { test, expect } from "@playwright/test";

test.describe("Dashboard", () => {
  test("should display app heading", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("text=Just Trade")).toBeVisible();
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

    // Switch to Recent Trades
    await page.click("text=Recent Trades");
    await expect(page.locator("text=Today's executed orders")).toBeVisible();

    // Switch to Watchlist
    await page.click("text=Watchlist");
    await expect(page.locator("text=META")).toBeVisible();
  });
});
