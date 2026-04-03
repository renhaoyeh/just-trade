import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { renderWithProviders } from "@/tests/render-helpers";
import Backtest from "./Backtest";

vi.mock("@/components/charts/BacktestChart", () => ({
  BacktestChart: () => <div data-testid="backtest-chart" />,
  CompareBacktestChart: ({ strategies }: { strategies: { name: string; enabled: boolean }[] }) => (
    <div data-testid="compare-chart">{strategies.filter((s: { enabled: boolean }) => s.enabled).length} strategies</div>
  ),
}));

const mockedInvoke = vi.mocked(invoke);

function renderBacktest() {
  return renderWithProviders(<Backtest />);
}

const mockBacktestResult = {
  symbol: "2330.TW",
  benchmark: { start_price: 500, end_price: 555, return_pct: 11.0, annualized_return_pct: 5.5 },
  metrics: {
    initial_capital: 1000000,
    total_invested: 1000000,
    final_equity: 1050000,
    total_return_pct: 5.0,
    annualized_return_pct: 2.5,
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
  prices: [
    { date: "2023-01-01", open: 500, high: 510, low: 495, close: 505, volume: 10000 },
    { date: "2023-06-01", open: 530, high: 540, low: 525, close: 535, volume: 12000 },
    { date: "2024-12-31", open: 550, high: 560, low: 545, close: 555, volume: 11000 },
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
    // Default SMA params
    expect(screen.getByDisplayValue("5")).toBeInTheDocument();
    expect(screen.getByDisplayValue("20")).toBeInTheDocument();
  });

  it("renders all strategy buttons", () => {
    renderBacktest();
    expect(screen.getByRole("button", { name: "均線交叉" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "RSI" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "布林通道" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "MACD" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "定期定額" })).toBeInTheDocument();
  });

  it("switches strategy params when clicking a different strategy", async () => {
    renderBacktest();

    // Click RSI
    await userEvent.click(screen.getByRole("button", { name: "RSI" }));
    expect(screen.getByDisplayValue("14")).toBeInTheDocument(); // period
    expect(screen.getByDisplayValue("70")).toBeInTheDocument(); // overbought
    expect(screen.getByDisplayValue("30")).toBeInTheDocument(); // oversold

    // Click MACD
    await userEvent.click(screen.getByRole("button", { name: "MACD" }));
    expect(screen.getByDisplayValue("12")).toBeInTheDocument(); // fast
    expect(screen.getByDisplayValue("26")).toBeInTheDocument(); // slow
    expect(screen.getByDisplayValue("9")).toBeInTheDocument();  // signal

    // Click Bollinger Bands
    await userEvent.click(screen.getByRole("button", { name: "布林通道" }));
    expect(screen.getByDisplayValue("2")).toBeInTheDocument(); // std_dev

    // Click DCA
    await userEvent.click(screen.getByRole("button", { name: "定期定額" }));
    expect(screen.getByDisplayValue("10000")).toBeInTheDocument(); // amount
    expect(screen.getByDisplayValue("22")).toBeInTheDocument(); // interval
  });

  it("calls run_backtest with SMA strategy and displays results", async () => {
    mockedInvoke.mockResolvedValueOnce(mockBacktestResult);
    renderBacktest();

    const runBtn = screen.getByRole("button", { name: "開始回測" });
    await userEvent.click(runBtn);

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith("run_backtest", {
        params: expect.objectContaining({
          symbol: "2330.TW",
          initial_capital: 1000000,
          strategy: { type: "SmaCrossover", short_period: 5, long_period: 20 },
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

  it("calls run_backtest with RSI strategy", async () => {
    mockedInvoke.mockResolvedValueOnce(mockBacktestResult);
    renderBacktest();

    await userEvent.click(screen.getByRole("button", { name: "RSI" }));
    await userEvent.click(screen.getByRole("button", { name: "開始回測" }));

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledWith("run_backtest", {
        params: expect.objectContaining({
          strategy: { type: "Rsi", period: 14, overbought: 70, oversold: 30 },
        }),
      });
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

  it("renders compare all button", () => {
    renderBacktest();
    expect(screen.getByRole("button", { name: "全部比較" })).toBeInTheDocument();
  });

  it("runs all strategies and shows comparison table", async () => {
    // Mock calls (one per strategy)
    mockedInvoke.mockResolvedValue(mockBacktestResult);
    renderBacktest();

    await userEvent.click(screen.getByRole("button", { name: "全部比較" }));

    await waitFor(() => {
      expect(mockedInvoke).toHaveBeenCalledTimes(12);
    });

    // Comparison table should appear
    await waitFor(() => {
      expect(screen.getByText("策略比較")).toBeInTheDocument();
      // All strategy names should be in the comparison table
      expect(screen.getAllByText("均線交叉").length).toBeGreaterThanOrEqual(1);
      expect(screen.getAllByText("RSI").length).toBeGreaterThanOrEqual(1);
      expect(screen.getAllByText("布林通道").length).toBeGreaterThanOrEqual(1);
      expect(screen.getAllByText("MACD").length).toBeGreaterThanOrEqual(1);
    });
  });
});
