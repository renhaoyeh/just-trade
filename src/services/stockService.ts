import { invoke } from "@tauri-apps/api/core";
import type {
  StockPrice,
  StockInfo,
  StockNews,
  SearchResult,
  Sector,
  SectorStock,
} from "@/types/stock";

export async function fetchStockHistory(
  symbol: string,
  startDate: string,
  endDate: string
): Promise<StockPrice[]> {
  return invoke("fetch_stock_history", { symbol, startDate, endDate });
}

export async function getStockInfo(symbol: string): Promise<StockInfo> {
  return invoke("get_stock_info", { symbol });
}

export async function fetchStockNews(
  symbol: string,
  count?: number
): Promise<StockNews[]> {
  return invoke("fetch_stock_news", { symbol, count });
}

export async function searchStocks(query: string): Promise<SearchResult[]> {
  return invoke("search_stocks", { query });
}

export async function getSectors(): Promise<Sector[]> {
  return invoke("get_sectors");
}

export async function getSectorStocks(sector: string): Promise<SectorStock[]> {
  return invoke("get_sector_stocks", { sector });
}
