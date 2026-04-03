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

// ---------------------------------------------------------------------------
// Fugle WebSocket
// ---------------------------------------------------------------------------

export interface FugleTrade {
  symbol: string;
  price: number | null;
  size: number | null;
  volume: number | null;
  bid: number | null;
  ask: number | null;
  time: string | null;
  serial: string | null;
  isOpen: boolean | null;
  isClose: boolean | null;
  isLimitUpPrice: boolean | null;
  isLimitDownPrice: boolean | null;
  isTrial: boolean | null;
}

export interface BookLevel {
  price: number | null;
  size: number | null;
}

export interface FugleBook {
  symbol: string;
  bids: BookLevel[];
  asks: BookLevel[];
  time: string | null;
}

export interface FugleCandle {
  symbol: string;
  open: number | null;
  high: number | null;
  low: number | null;
  close: number | null;
  volume: number | null;
  average: number | null;
  time: string | null;
}

export interface FugleWsStatus {
  connected: boolean;
  message: string;
}

// ---------------------------------------------------------------------------
// Backtest
// ---------------------------------------------------------------------------

export interface BacktestTrade {
  date: string;
  action: string;
  price: number;
  shares: number;
  cost: number;
  pnl: number;
  balance: number;
}

export interface EquityPoint {
  date: string;
  equity: number;
  cash: number;
  position_value: number;
}

export interface BacktestMetrics {
  initial_capital: number;
  final_equity: number;
  total_return_pct: number;
  max_drawdown_pct: number;
  total_trades: number;
  winning_trades: number;
  losing_trades: number;
  win_rate_pct: number;
  total_commission: number;
  total_tax: number;
  start_date: string;
  end_date: string;
  trading_days: number;
}

export interface BacktestResult {
  symbol: string;
  metrics: BacktestMetrics;
  trades: BacktestTrade[];
  equity_curve: EquityPoint[];
}
