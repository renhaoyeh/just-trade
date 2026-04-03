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

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

export interface AppSettings {
  llm_provider: string | null;
  llm_model: string | null;
  openai_api_key: string | null;
  anthropic_api_key: string | null;
  google_api_key: string | null;
  groq_api_key: string | null;
  ollama_base_url: string | null;
}

export async function getSettings(): Promise<AppSettings> {
  return invoke("get_settings");
}

export async function saveSettings(settings: AppSettings): Promise<void> {
  return invoke("save_settings", { settings });
}

export async function getLlmModels(provider: string): Promise<string[]> {
  return invoke("llm_models", { provider });
}
