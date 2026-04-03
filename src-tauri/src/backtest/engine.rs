use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::models::stock_price::StockPrice;

use super::strategy::Signal;

/// Lightweight price bar for frontend charting (no DB fields).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceBar {
    pub date: NaiveDate,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: i64,
}

/// Taiwan stock trading costs.
const TW_COMMISSION_RATE: f64 = 0.001425; // 0.1425% each way
const TW_TAX_RATE: f64 = 0.003; // 0.3% on sell

/// Configuration for the backtest engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestConfig {
    pub initial_capital: f64,
    pub commission_rate: f64,
    pub tax_rate: f64,
}

impl Default for BacktestConfig {
    fn default() -> Self {
        Self {
            initial_capital: 1_000_000.0,
            commission_rate: TW_COMMISSION_RATE,
            tax_rate: TW_TAX_RATE,
        }
    }
}

/// A single trade record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub date: NaiveDate,
    pub action: String, // "buy" | "sell"
    pub price: f64,
    pub shares: i64,
    pub cost: f64,    // commission + tax
    pub pnl: f64,     // realized P&L for sells
    pub balance: f64,  // cash after trade
}

/// A daily equity snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquityPoint {
    pub date: NaiveDate,
    pub equity: f64,
    pub cash: f64,
    pub position_value: f64,
}

/// Summary metrics for a completed backtest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestMetrics {
    pub initial_capital: f64,
    /// Total amount actually spent on buying shares (excluding commission).
    pub total_invested: f64,
    pub final_equity: f64,
    pub total_return_pct: f64,
    pub max_drawdown_pct: f64,
    pub total_trades: usize,
    pub winning_trades: usize,
    pub losing_trades: usize,
    pub win_rate_pct: f64,
    pub total_commission: f64,
    pub total_tax: f64,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub trading_days: usize,
}

/// Full backtest result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestResult {
    pub symbol: String,
    pub metrics: BacktestMetrics,
    pub trades: Vec<Trade>,
    pub equity_curve: Vec<EquityPoint>,
    pub prices: Vec<PriceBar>,
}

/// Run a backtest given price data and pre-computed signals.
pub fn run_backtest(
    symbol: &str,
    prices: &[StockPrice],
    signals: &[Signal],
    config: &BacktestConfig,
) -> BacktestResult {
    assert_eq!(prices.len(), signals.len());

    let mut cash = config.initial_capital;
    let mut shares: i64 = 0;
    let mut avg_cost: f64 = 0.0;
    let mut trades: Vec<Trade> = Vec::new();
    let mut equity_curve: Vec<EquityPoint> = Vec::new();
    let mut total_commission = 0.0;
    let mut total_tax = 0.0;
    let mut total_invested = 0.0;
    let mut is_dca = false;

    for (i, (price, signal)) in prices.iter().zip(signals.iter()).enumerate() {
        match signal {
            Signal::Buy if shares == 0 => {
                // Buy as many shares as we can afford (支援零股)
                let buy_shares = (cash / (price.close * (1.0 + config.commission_rate))) as i64;
                if buy_shares > 0 {
                    let amount = price.close * buy_shares as f64;
                    let commission = (amount * config.commission_rate).max(20.0);
                    total_commission += commission;
                    total_invested += amount;
                    cash -= amount + commission;
                    avg_cost = price.close;
                    shares = buy_shares;

                    trades.push(Trade {
                        date: price.date,
                        action: "buy".into(),
                        price: price.close,
                        shares: buy_shares,
                        cost: commission,
                        pnl: 0.0,
                        balance: cash,
                    });
                }
            }
            Signal::BuyFixed(budget) => {
                // DCA: buy a fixed dollar amount worth of shares
                is_dca = true;
                let spend = budget.min(cash);
                let buy_shares = (spend / (price.close * (1.0 + config.commission_rate))) as i64;
                if buy_shares > 0 {
                    let amount = price.close * buy_shares as f64;
                    let commission = (amount * config.commission_rate).max(20.0);
                    total_commission += commission;
                    total_invested += amount;
                    cash -= amount + commission;
                    // Weighted average cost
                    let total_cost = avg_cost * shares as f64 + price.close * buy_shares as f64;
                    shares += buy_shares;
                    avg_cost = total_cost / shares as f64;

                    trades.push(Trade {
                        date: price.date,
                        action: "buy".into(),
                        price: price.close,
                        shares: buy_shares,
                        cost: commission,
                        pnl: 0.0,
                        balance: cash,
                    });
                }
            }
            Signal::Sell if shares > 0 => {
                let amount = price.close * shares as f64;
                let commission = (amount * config.commission_rate).max(20.0);
                let tax = amount * config.tax_rate;
                let pnl = (price.close - avg_cost) * shares as f64 - commission - tax;
                total_commission += commission;
                total_tax += tax;
                cash += amount - commission - tax;

                trades.push(Trade {
                    date: price.date,
                    action: "sell".into(),
                    price: price.close,
                    shares,
                    cost: commission + tax,
                    pnl,
                    balance: cash,
                });

                shares = 0;
                avg_cost = 0.0;
            }
            _ => {}
        }

        // Record daily equity
        let position_value = shares as f64 * price.close;
        equity_curve.push(EquityPoint {
            date: price.date,
            equity: cash + position_value,
            cash,
            position_value,
        });

        // Force sell on last day if still holding
        if i == prices.len() - 1 && shares > 0 {
            let amount = price.close * shares as f64;
            let commission = (amount * config.commission_rate).max(20.0);
            let tax = amount * config.tax_rate;
            let pnl = (price.close - avg_cost) * shares as f64 - commission - tax;
            total_commission += commission;
            total_tax += tax;
            cash += amount - commission - tax;

            trades.push(Trade {
                date: price.date,
                action: "sell".into(),
                price: price.close,
                shares,
                cost: commission + tax,
                pnl,
                balance: cash,
            });

            shares = 0;

            // Update last equity point
            if let Some(last) = equity_curve.last_mut() {
                last.equity = cash;
                last.cash = cash;
                last.position_value = 0.0;
            }
        }
    }

    // Compute metrics
    let final_equity = equity_curve.last().map(|e| e.equity).unwrap_or(config.initial_capital);
    // DCA: return based on actual invested amount (not idle cash)
    // All-in strategies: return based on initial capital (same money recycled)
    let profit = final_equity - config.initial_capital;
    let return_base = if is_dca && total_invested > 0.0 {
        total_invested
    } else {
        config.initial_capital
    };
    let total_return_pct = if return_base > 0.0 {
        profit / return_base * 100.0
    } else {
        0.0
    };
    let max_drawdown_pct = compute_max_drawdown(&equity_curve);

    let sell_trades: Vec<&Trade> = trades.iter().filter(|t| t.action == "sell").collect();
    let winning = sell_trades.iter().filter(|t| t.pnl > 0.0).count();
    let losing = sell_trades.iter().filter(|t| t.pnl <= 0.0).count();
    let total_sell_trades = sell_trades.len();
    let win_rate = if total_sell_trades > 0 {
        winning as f64 / total_sell_trades as f64 * 100.0
    } else {
        0.0
    };

    let metrics = BacktestMetrics {
        initial_capital: config.initial_capital,
        total_invested,
        final_equity,
        total_return_pct,
        max_drawdown_pct,
        total_trades: trades.len(),
        winning_trades: winning,
        losing_trades: losing,
        win_rate_pct: win_rate,
        total_commission,
        total_tax,
        start_date: prices.first().map(|p| p.date).unwrap_or_default(),
        end_date: prices.last().map(|p| p.date).unwrap_or_default(),
        trading_days: prices.len(),
    };

    let price_bars = prices
        .iter()
        .map(|p| PriceBar {
            date: p.date,
            open: p.open,
            high: p.high,
            low: p.low,
            close: p.close,
            volume: p.volume,
        })
        .collect();

    BacktestResult {
        symbol: symbol.to_string(),
        metrics,
        trades,
        equity_curve,
        prices: price_bars,
    }
}

/// Compute maximum drawdown percentage from equity curve.
fn compute_max_drawdown(curve: &[EquityPoint]) -> f64 {
    let mut peak = f64::MIN;
    let mut max_dd = 0.0_f64;

    for point in curve {
        if point.equity > peak {
            peak = point.equity;
        }
        let dd = (peak - point.equity) / peak * 100.0;
        if dd > max_dd {
            max_dd = dd;
        }
    }

    max_dd
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn make_prices(closes: &[f64]) -> Vec<StockPrice> {
        closes
            .iter()
            .enumerate()
            .map(|(i, &c)| StockPrice {
                id: i as i64,
                symbol: "2330.TW".into(),
                date: NaiveDate::from_ymd_opt(2024, 1, 1)
                    .unwrap()
                    .checked_add_signed(chrono::Duration::days(i as i64))
                    .unwrap(),
                open: c,
                high: c,
                low: c,
                close: c,
                volume: 1000,
                adj_close: Some(c),
                created_at: String::new(),
            })
            .collect()
    }

    #[test]
    fn test_buy_and_sell() {
        let prices = make_prices(&[100.0, 110.0, 105.0]);
        let signals = vec![Signal::Buy, Signal::Hold, Signal::Sell];
        let config = BacktestConfig {
            initial_capital: 1_000_000.0,
            ..Default::default()
        };

        let result = run_backtest("TEST", &prices, &signals, &config);
        assert_eq!(result.trades.len(), 2);
        assert_eq!(result.trades[0].action, "buy");
        assert_eq!(result.trades[1].action, "sell");
        assert_eq!(result.equity_curve.len(), 3);
    }

    #[test]
    fn test_no_signals() {
        let prices = make_prices(&[100.0, 110.0, 120.0]);
        let signals = vec![Signal::Hold, Signal::Hold, Signal::Hold];
        let config = BacktestConfig::default();

        let result = run_backtest("TEST", &prices, &signals, &config);
        assert!(result.trades.is_empty());
        assert!((result.metrics.total_return_pct).abs() < 1e-9);
    }

    #[test]
    fn test_force_sell_on_last_day() {
        let prices = make_prices(&[100.0, 110.0, 120.0]);
        let signals = vec![Signal::Buy, Signal::Hold, Signal::Hold];
        let config = BacktestConfig::default();

        let result = run_backtest("TEST", &prices, &signals, &config);
        // Should have buy + forced sell
        assert_eq!(result.trades.len(), 2);
        assert_eq!(result.trades[1].action, "sell");
    }

    #[test]
    fn test_max_drawdown() {
        let curve = vec![
            EquityPoint { date: NaiveDate::default(), equity: 100.0, cash: 100.0, position_value: 0.0 },
            EquityPoint { date: NaiveDate::default(), equity: 120.0, cash: 120.0, position_value: 0.0 },
            EquityPoint { date: NaiveDate::default(), equity: 90.0, cash: 90.0, position_value: 0.0 },
            EquityPoint { date: NaiveDate::default(), equity: 110.0, cash: 110.0, position_value: 0.0 },
        ];
        let dd = compute_max_drawdown(&curve);
        // Peak = 120, trough = 90 → dd = 25%
        assert!((dd - 25.0).abs() < 1e-9);
    }

    #[test]
    fn test_dca_multiple_buys() {
        // 5 days, DCA buys on day 0 and day 2 (interval=2)
        let prices = make_prices(&[100.0, 105.0, 110.0, 108.0, 115.0]);
        let signals = vec![
            Signal::BuyFixed(50000.0),
            Signal::Hold,
            Signal::BuyFixed(50000.0),
            Signal::Hold,
            Signal::Hold,
        ];
        let config = BacktestConfig {
            initial_capital: 200_000.0,
            ..Default::default()
        };

        let result = run_backtest("TEST", &prices, &signals, &config);
        // 2 DCA buys + 1 forced sell
        let buy_count = result.trades.iter().filter(|t| t.action == "buy").count();
        let sell_count = result.trades.iter().filter(|t| t.action == "sell").count();
        assert_eq!(buy_count, 2);
        assert_eq!(sell_count, 1);
        // Should have accumulated shares from both buys
        assert!(result.trades.last().unwrap().shares > result.trades[0].shares);
    }
}
