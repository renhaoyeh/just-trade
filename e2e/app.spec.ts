import { test, expect } from "@playwright/test";

test.describe("App", () => {
  test("should display welcome heading", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("h1")).toHaveText("Welcome to Tauri + React");
  });

  test("should render logos and description", async ({ page }) => {
    await page.goto("/");

    await expect(page.locator('img[alt="Vite logo"]')).toBeVisible();
    await expect(page.locator('img[alt="React logo"]')).toBeVisible();
    await expect(
      page.locator("text=Click on the Tauri, Vite, and React logos")
    ).toBeVisible();
  });

  test("should have a greet form with input and button", async ({ page }) => {
    await page.goto("/");

    const input = page.locator('input[placeholder="Enter a name..."]');
    const button = page.locator('button[type="submit"]');

    await expect(input).toBeVisible();
    await expect(button).toHaveText("Greet");
  });
});
