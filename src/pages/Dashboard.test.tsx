import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { renderWithProviders } from "@/tests/render-helpers";
import Dashboard from "./Dashboard";

function renderDashboard() {
  return renderWithProviders(<Dashboard />);
}

const mockedInvoke = vi.mocked(invoke);

const mockStockPrice = (date: string, close: number) => ({
  id: 1,
  symbol: "2330.TW",
  date,
  open: close - 5,
  high: close + 5,
  low: close - 10,
  close,
  volume: 25000000,
  adj_close: close,
  created_at: `${date}T00:00:00Z`,
});

describe("Dashboard - Range Selector", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders all range option buttons", () => {
    renderDashboard();
    for (const label of ["1M", "3M", "6M", "1Y", "5Y"]) {
      expect(screen.getByRole("button", { name: label })).toBeInTheDocument();
    }
  });

  it("defaults to 3M range", () => {
    renderDashboard();
    expect(screen.getByText(/近 3 個月/)).toBeInTheDocument();
  });

  it("updates description when range changes", async () => {
    const user = userEvent.setup();
    renderDashboard();

    await user.click(screen.getByRole("button", { name: "1Y" }));
    expect(screen.getByText(/近 1 年/)).toBeInTheDocument();
  });

  it("calls fetchStockHistory with correct date range for 1M", async () => {
    const user = userEvent.setup();
    renderDashboard();

    mockedInvoke.mockClear();
    await user.click(screen.getByRole("button", { name: "1M" }));

    const historyCall = mockedInvoke.mock.calls.find(
      (call) => call[0] === "fetch_stock_history"
    );
    expect(historyCall).toBeDefined();
    const args = historyCall![1] as { startDate: string; endDate: string };
    const start = new Date(args.startDate);
    const end = new Date(args.endDate);
    const diffDays = Math.round(
      (end.getTime() - start.getTime()) / (1000 * 60 * 60 * 24)
    );
    expect(diffDays).toBe(30);
  });

  it("calls fetchStockHistory with correct date range for 1Y", async () => {
    const user = userEvent.setup();
    renderDashboard();

    mockedInvoke.mockClear();
    await user.click(screen.getByRole("button", { name: "1Y" }));

    const historyCall = mockedInvoke.mock.calls.find(
      (call) => call[0] === "fetch_stock_history"
    );
    expect(historyCall).toBeDefined();
    const args = historyCall![1] as { startDate: string; endDate: string };
    const start = new Date(args.startDate);
    const end = new Date(args.endDate);
    const diffDays = Math.round(
      (end.getTime() - start.getTime()) / (1000 * 60 * 60 * 24)
    );
    expect(diffDays).toBe(365);
  });
});

describe("Dashboard - Chart Empty/Loading States", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("shows chart card even when no data is loaded", () => {
    renderDashboard();
    expect(screen.getByText(/股價走勢/)).toBeInTheDocument();
  });

  it("shows empty message when data is empty and not loading", async () => {
    mockedInvoke.mockResolvedValue([]);
    renderDashboard();
    expect(await screen.findByText("無股價資料")).toBeInTheDocument();
  });

  it("shows chart when data is available", async () => {
    const prices = [
      mockStockPrice("2026-03-01", 600),
      mockStockPrice("2026-03-02", 610),
    ];
    mockedInvoke.mockImplementation(async (cmd) => {
      if (cmd === "fetch_stock_history") return prices;
      if (cmd === "fetch_stock_news") return [];
      return {};
    });
    renderDashboard();
    const chartCard = await screen.findByText(/股價走勢/);
    expect(chartCard).toBeInTheDocument();
    expect(screen.queryByText("無股價資料")).not.toBeInTheDocument();
  });
});

describe("Dashboard - i18n", () => {
  it("renders tab labels from i18n", () => {
    renderDashboard();
    expect(screen.getByRole("tab", { name: "自選清單" })).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: "歷史股價" })).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: "相關新聞" })).toBeInTheDocument();
  });

  it("renders search button from i18n", () => {
    renderDashboard();
    expect(screen.getByRole("button", { name: "查詢" })).toBeInTheDocument();
  });

  it("renders search placeholder from i18n", () => {
    renderDashboard();
    expect(
      screen.getByPlaceholderText("輸入股票代號 (如 2330)")
    ).toBeInTheDocument();
  });
});
