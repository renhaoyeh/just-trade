use serde::{Deserialize, Serialize};

use crate::models::stock_price::StockPrice;

/// A trading signal produced by a strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Signal {
    Buy,
    Sell,
    Hold,
}

/// Configuration for the SMA-crossover strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmaCrossoverConfig {
    pub short_period: usize,
    pub long_period: usize,
}

impl Default for SmaCrossoverConfig {
    fn default() -> Self {
        Self {
            short_period: 5,
            long_period: 20,
        }
    }
}

/// Compute simple moving averages and produce signals.
///
/// Signal logic:
/// - When short SMA crosses **above** long SMA → Buy
/// - When short SMA crosses **below** long SMA → Sell
/// - Otherwise → Hold
pub fn sma_crossover_signals(prices: &[StockPrice], config: &SmaCrossoverConfig) -> Vec<Signal> {
    let n = prices.len();
    if n < config.long_period {
        return vec![Signal::Hold; n];
    }

    let closes: Vec<f64> = prices.iter().map(|p| p.close).collect();
    let short_sma = rolling_sma(&closes, config.short_period);
    let long_sma = rolling_sma(&closes, config.long_period);

    let mut signals = vec![Signal::Hold; n];

    for i in 1..n {
        let (Some(s_prev), Some(l_prev)) = (short_sma[i - 1], long_sma[i - 1]) else {
            continue;
        };
        let (Some(s_cur), Some(l_cur)) = (short_sma[i], long_sma[i]) else {
            continue;
        };

        if s_prev <= l_prev && s_cur > l_cur {
            signals[i] = Signal::Buy;
        } else if s_prev >= l_prev && s_cur < l_cur {
            signals[i] = Signal::Sell;
        }
    }

    signals
}

/// Rolling simple moving average. Returns `None` for indices where there is
/// not yet enough data.
fn rolling_sma(data: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = data.len();
    let mut result = vec![None; n];
    if period == 0 || n < period {
        return result;
    }

    let mut sum: f64 = data[..period].iter().sum();
    result[period - 1] = Some(sum / period as f64);

    for i in period..n {
        sum += data[i] - data[i - period];
        result[i] = Some(sum / period as f64);
    }

    result
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
                symbol: "TEST".into(),
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
    fn test_rolling_sma() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let sma = rolling_sma(&data, 3);
        assert_eq!(sma[0], None);
        assert_eq!(sma[1], None);
        assert!((sma[2].unwrap() - 2.0).abs() < 1e-9);
        assert!((sma[3].unwrap() - 3.0).abs() < 1e-9);
        assert!((sma[4].unwrap() - 4.0).abs() < 1e-9);
    }

    #[test]
    fn test_sma_crossover_not_enough_data() {
        let prices = make_prices(&[10.0, 20.0, 30.0]);
        let config = SmaCrossoverConfig {
            short_period: 2,
            long_period: 5,
        };
        let signals = sma_crossover_signals(&prices, &config);
        assert!(signals.iter().all(|s| *s == Signal::Hold));
    }

    #[test]
    fn test_sma_crossover_buy_signal() {
        // Build a price series where short SMA crosses above long SMA
        // Long period = 3, short period = 2
        // Prices: descending then sharply rising → crossover
        let prices = make_prices(&[50.0, 40.0, 30.0, 20.0, 10.0, 50.0, 80.0]);
        let config = SmaCrossoverConfig {
            short_period: 2,
            long_period: 3,
        };
        let signals = sma_crossover_signals(&prices, &config);
        assert!(signals.contains(&Signal::Buy));
    }
}
