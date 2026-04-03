import { test, expect } from "@playwright/test";

test.describe("Dashboard", () => {
  test("should display app heading and sidebar", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("text=Just Trade").first()).toBeVisible();
    await expect(page.locator('[data-slot="sidebar"]')).toBeVisible();
    await expect(page.locator("text=Dashboard").first()).toBeVisible();
  });

  test("should render stock search input", async ({ page }) => {
    await page.goto("/");
    const searchInput = page.getByPlaceholder("輸入股票代號 (如 2330)");
    await expect(searchInput).toBeVisible();
  });

  test("should show default symbol badge", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("text=2330.TW").first()).toBeVisible();
  });

  test("should render stat cards", async ({ page }) => {
    await page.goto("/");
    // These stats are present even with loading/empty state
    await expect(page.locator("text=52 Week High")).toBeVisible();
    await expect(page.locator("text=52 Week Low")).toBeVisible();
    await expect(page.locator("text=Market Cap")).toBeVisible();
  });

  test("should switch between tabs", async ({ page }) => {
    await page.goto("/");

    // Default tab is watchlist (自選清單)
    await expect(
      page.locator('[data-slot="tabs-trigger"]', { hasText: "自選清單" })
    ).toBeVisible();

    // Switch to history tab (歷史股價)
    await page
      .locator('[data-slot="tabs-trigger"]', { hasText: "歷史股價" })
      .click();
    await expect(
      page.locator("text=歷史股價").first()
    ).toBeVisible();

    // Switch to news tab (相關新聞)
    await page
      .locator('[data-slot="tabs-trigger"]', { hasText: "相關新聞" })
      .click();
    await expect(
      page.locator("text=相關新聞").first()
    ).toBeVisible();
  });

  test("should search for a stock symbol", async ({ page }) => {
    await page.goto("/");
    const searchInput = page.getByPlaceholder("輸入股票代號 (如 2330)");

    // Type a symbol and press Enter
    await searchInput.fill("2317");
    await searchInput.press("Enter");

    // Badge should update to new symbol
    await expect(page.locator("text=2317.TW").first()).toBeVisible();
  });

  test("should have sidebar navigation items", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("text=Dashboard").first()).toBeVisible();
    await expect(page.locator("text=Portfolio").first()).toBeVisible();
    await expect(page.locator("text=Trade").first()).toBeVisible();
  });
});
