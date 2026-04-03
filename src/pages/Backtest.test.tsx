import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { renderWithProviders } from "@/tests/render-helpers";
import Backtest from "./Backtest";

const mockedInvoke = vi.mocked(invoke);

function renderBacktest() {
  return renderWithProviders(<Backtest />);
}

const mockBacktestResult = {
  symbol: "2330.TW",
  metrics: {
    initial_capital: 1000000,
    final_equity: 1050000,
    total_return_pct: 5.0,
    max_drawdown_pct: 3.2,
    total_trades: 4,
    winning_trades: 2,
    losing_trades: 1,
    win_rate_pct: 66.7,
    total_commission: 2850,
    total_tax: 3000,
    start_date: "2023-01-01",
    end_date: "2024-12-31",
    trading_days: 480,
  },
  trades: [
    {
      date: "2023-02-15",
      action: "buy",
      price: 520.0,
      shares: 1000,
      cost: 741,
      pnl: 0,
      balance: 479259,
    },
    {
      date: "2023-05-10",
      action: "sell",
      price: 550.0,
      shares: 1000,
      cost: 2433,
      pnl: 27567,
      balance: 1027567,
    },
  ],
  equity_curve: [
    { date: "2023-01-01", equity: 1000000, cash: 1000000, position_value: 0 },
    { date: "2023-06-01", equity: 1025000, cash: 1025000, position_value: 0 },
    { date: "2024-12-31", equity: 1050000, cash: 1050000, position_value: 0 },
  ],
};

describe("Backtest Page", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders the config form with all inputs", () => {
    renderBacktest();
    expect(screen.getByText("策略回測")).toBeInTheDocument();
    expect(screen.getByText("回測設定")).toBeInTheDocument();
    expect(screen.getByDisplayValue("2330.TW")).toBeInTheDocument();
    expect(screen.getByDisplayValue("1000000")).toBeInTheDocument();
    expect(screen.getByDisplayValue("5")).toBeInTheDocument();
    expect(screen.getByDisplayValue("20")).toBeInTheDocument();
  });

  it("renders the run button", () => {
    renderBacktest();
    expect(screen.getByRole("button", { name: "開始回測" })).toBeInTheDocument();
  });

  it("calls run_backtest and displays results on submit", async () => {
    mockedInvoke.mockResolvedValueOnce(mockBacktestResult);
    renderBacktest();

    const runBtn = screen.getByRole("button", { name: "開始回測" });
    await userEvent.click(runBtn);

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith("run_backtest", {
        params: expect.objectContaining({
          symbol: "2330.TW",
          initial_capital: 1000000,
          short_period: 5,
          long_period: 20,
        }),
      });
    });

    // Metrics should appear
    await waitFor(() => {
      expect(screen.getByText("+5.00%")).toBeInTheDocument();
      expect(screen.getByText("-3.20%")).toBeInTheDocument();
      expect(screen.getByText("66.7%")).toBeInTheDocument();
    });
  });

  it("displays error when backtest fails", async () => {
    mockedInvoke.mockRejectedValueOnce("Not enough data");
    renderBacktest();

    const runBtn = screen.getByRole("button", { name: "開始回測" });
    await userEvent.click(runBtn);

    await waitFor(() => {
      expect(screen.getByText("Not enough data")).toBeInTheDocument();
    });
  });

  it("shows trade history table after successful backtest", async () => {
    mockedInvoke.mockResolvedValueOnce(mockBacktestResult);
    renderBacktest();

    await userEvent.click(screen.getByRole("button", { name: "開始回測" }));

    await waitFor(() => {
      expect(screen.getByText("交易紀錄")).toBeInTheDocument();
      expect(screen.getByText("2023-02-15")).toBeInTheDocument();
      expect(screen.getByText("2023-05-10")).toBeInTheDocument();
    });
  });
});
