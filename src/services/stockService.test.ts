import { describe, it, expect, vi, beforeEach } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import {
  fetchStockHistory,
  getStockInfo,
  fetchStockNews,
  searchStocks,
} from "./stockService";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const mockedInvoke = vi.mocked(invoke);

describe("stockService", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("fetchStockHistory", () => {
    it("calls invoke with correct command and params", async () => {
      mockedInvoke.mockResolvedValue([]);
      await fetchStockHistory("2330.TW", "2026-01-01", "2026-03-01");
      expect(mockedInvoke).toHaveBeenCalledWith("fetch_stock_history", {
        symbol: "2330.TW",
        startDate: "2026-01-01",
        endDate: "2026-03-01",
      });
    });

    it("returns stock prices from backend", async () => {
      const mockPrices = [
        {
          id: 1,
          symbol: "2330.TW",
          date: "2026-01-02",
          open: 600.0,
          high: 610.0,
          low: 595.0,
          close: 605.0,
          volume: 25000000,
          adj_close: 605.0,
          created_at: "2026-01-02T00:00:00Z",
        },
      ];
      mockedInvoke.mockResolvedValue(mockPrices);
      const result = await fetchStockHistory("2330.TW", "2026-01-01", "2026-01-03");
      expect(result).toEqual(mockPrices);
    });

    it("propagates errors from backend", async () => {
      mockedInvoke.mockRejectedValue("DB error: connection refused");
      await expect(
        fetchStockHistory("2330.TW", "2026-01-01", "2026-03-01")
      ).rejects.toBe("DB error: connection refused");
    });
  });

  describe("getStockInfo", () => {
    it("calls invoke with correct command and params", async () => {
      mockedInvoke.mockResolvedValue({});
      await getStockInfo("2330.TW");
      expect(mockedInvoke).toHaveBeenCalledWith("get_stock_info", {
        symbol: "2330.TW",
      });
    });

    it("returns stock info from backend", async () => {
      const mockInfo = {
        symbol: "2330.TW",
        short_name: "TSMC",
        long_name: "Taiwan Semiconductor Manufacturing",
        market_cap: 20000000000000,
        pe_ratio: 25.5,
        forward_pe: 22.0,
        dividend_yield: 0.015,
        fifty_two_week_high: 700.0,
        fifty_two_week_low: 500.0,
        fifty_day_average: 620.0,
        two_hundred_day_average: 580.0,
        currency: "TWD",
        exchange: "TAI",
      };
      mockedInvoke.mockResolvedValue(mockInfo);
      const result = await getStockInfo("2330.TW");
      expect(result).toEqual(mockInfo);
    });
  });

  describe("fetchStockNews", () => {
    it("calls invoke with correct command and params", async () => {
      mockedInvoke.mockResolvedValue([]);
      await fetchStockNews("2330.TW", 5);
      expect(mockedInvoke).toHaveBeenCalledWith("fetch_stock_news", {
        symbol: "2330.TW",
        count: 5,
      });
    });

    it("defaults count to undefined when not provided", async () => {
      mockedInvoke.mockResolvedValue([]);
      await fetchStockNews("2330.TW");
      expect(mockedInvoke).toHaveBeenCalledWith("fetch_stock_news", {
        symbol: "2330.TW",
        count: undefined,
      });
    });
  });

  describe("searchStocks", () => {
    it("calls invoke with correct command and params", async () => {
      mockedInvoke.mockResolvedValue([]);
      await searchStocks("台積電");
      expect(mockedInvoke).toHaveBeenCalledWith("search_stocks", {
        query: "台積電",
      });
    });

    it("returns search results from backend", async () => {
      const mockResults = [
        {
          symbol: "2330.TW",
          short_name: "TSMC",
          exchange: "TAI",
          quote_type: "EQUITY",
        },
      ];
      mockedInvoke.mockResolvedValue(mockResults);
      const result = await searchStocks("2330");
      expect(result).toEqual(mockResults);
    });
  });
});
