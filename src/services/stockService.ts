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

export interface FugleSettings {
  api_key: string | null;
}

export interface AppSettings {
  openai: ProviderSettings | null;
  anthropic: ProviderSettings | null;
  google: ProviderSettings | null;
  groq: ProviderSettings | null;
  ollama: ProviderSettings | null;
  fugle: FugleSettings | null;
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

export interface AnalysisRecord {
  id: number;
  symbol: string;
  signal: string;
  created_at: string;
  report_path: string;
}

export async function getAnalysisHistory(limit?: number): Promise<AnalysisRecord[]> {
  return invoke("get_analysis_history", { limit: limit ?? 20 });
}

export async function getAnalysisDetail(id: number): Promise<unknown> {
  return invoke("get_analysis_detail", { id });
}

export async function getLlmModels(provider: string): Promise<string[]> {
  return invoke("llm_models", { provider });
}

// ---------------------------------------------------------------------------
// Fugle WebSocket
// ---------------------------------------------------------------------------

export async function fugleWsConnect(apiKey: string): Promise<void> {
  return invoke("fugle_ws_connect", { apiKey });
}

export async function fugleWsDisconnect(): Promise<void> {
  return invoke("fugle_ws_disconnect");
}

export async function fugleWsSubscribe(
  channel: string,
  symbol: string
): Promise<void> {
  return invoke("fugle_ws_subscribe", { channel, symbol });
}

export async function fugleWsUnsubscribe(
  channel: string,
  symbol: string
): Promise<void> {
  return invoke("fugle_ws_unsubscribe", { channel, symbol });
}

export async function fugleWsStatus(): Promise<boolean> {
  return invoke("fugle_ws_status");
}

// ---------------------------------------------------------------------------
// Backtest
// ---------------------------------------------------------------------------

import type { BacktestResult } from "@/types/stock";

export type StrategyConfig =
  | { type: "SmaCrossover"; short_period: number; long_period: number }
  | { type: "Rsi"; period: number; overbought: number; oversold: number }
  | { type: "BollingerBands"; period: number; std_dev: number }
  | { type: "Macd"; fast_period: number; slow_period: number; signal_period: number }
  | { type: "Dca"; amount: number; interval_days: number };

export interface RunBacktestParams {
  symbol: string;
  start_date: string;
  end_date: string;
  initial_capital: number;
  strategy: StrategyConfig;
}

export async function runBacktest(
  params: RunBacktestParams
): Promise<BacktestResult> {
  return invoke("run_backtest", { params });
}
