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

export interface ProviderSettings {
  api_key: string | null;
  model: string | null;
  base_url: string | null;
}

export interface AppSettings {
  openai: ProviderSettings | null;
  anthropic: ProviderSettings | null;
  google: ProviderSettings | null;
  groq: ProviderSettings | null;
  ollama: ProviderSettings | null;
}

export async function getSettings(): Promise<AppSettings> {
  return invoke("get_settings");
}

export async function saveSettings(settings: AppSettings): Promise<void> {
  return invoke("save_settings", { settings });
}

export interface LlmConfig {
  provider: string;
  model: string;
  api_key?: string | null;
  base_url?: string | null;
  temperature?: number | null;
  max_tokens?: number | null;
  reasoning_effort?: string | null;
  effort?: string | null;
  thinking_level?: string | null;
}

export type GroupedModels = Record<string, string[]>;

export async function testLlmConnection(config: LlmConfig): Promise<GroupedModels> {
  return invoke("llm_test", { config });
}

export interface PipelineConfig {
  quick_llm: LlmConfig;
  deep_llm: LlmConfig;
  max_debate_rounds: number;
  max_risk_rounds: number;
  enable_market_analyst: boolean;
  enable_news_analyst: boolean;
  enable_fundamentals_analyst: boolean;
  enable_social_analyst: boolean;
  cooldown_secs: number;
}

export async function runAnalysis(
  symbol: string,
  pipelineConfig: PipelineConfig
): Promise<unknown> {
  return invoke("run_analysis", { symbol, pipelineConfig });
}

export async function getLlmModels(provider: string): Promise<string[]> {
  return invoke("llm_models", { provider });
}
