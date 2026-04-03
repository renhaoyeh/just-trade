export interface StockPrice {
  id: number;
  symbol: string;
  date: string;
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
  adj_close: number | null;
  created_at: string;
}

export interface StockInfo {
  symbol: string;
  short_name: string | null;
  long_name: string | null;
  market_cap: number | null;
  pe_ratio: number | null;
  forward_pe: number | null;
  dividend_yield: number | null;
  fifty_two_week_high: number | null;
  fifty_two_week_low: number | null;
  fifty_day_average: number | null;
  two_hundred_day_average: number | null;
  currency: string | null;
  exchange: string | null;
}

export interface StockNews {
  title: string;
  summary: string | null;
  publisher: string;
  link: string | null;
  pub_date: string | null;
}

export interface SearchResult {
  symbol: string;
  short_name: string | null;
  exchange: string | null;
  quote_type: string | null;
}

export interface Sector {
  name: string;
  stock_count: number;
}

export interface SectorStock {
  symbol: string;
  name: string;
  sector: string;
  closing_price: number | null;
  change: number | null;
  trade_volume: number | null;
}
