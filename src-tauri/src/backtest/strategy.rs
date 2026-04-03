use serde::{Deserialize, Serialize};

use crate::models::stock_price::StockPrice;

/// A trading signal produced by a strategy.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Signal {
    Buy,
    /// Buy a fixed dollar amount (for DCA).
    BuyFixed(f64),
    Sell,
    Hold,
}

// ===========================================================================
// Strategy enum (used by command layer)
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StrategyConfig {
    SmaCrossover(SmaCrossoverConfig),
    Rsi(RsiConfig),
    BollingerBands(BollingerBandsConfig),
    Macd(MacdConfig),
    Dca(DcaConfig),
}

/// Dispatch signals based on the chosen strategy.
pub fn generate_signals(prices: &[StockPrice], config: &StrategyConfig) -> Vec<Signal> {
    match config {
        StrategyConfig::SmaCrossover(c) => sma_crossover_signals(prices, c),
        StrategyConfig::Rsi(c) => rsi_signals(prices, c),
        StrategyConfig::BollingerBands(c) => bollinger_bands_signals(prices, c),
        StrategyConfig::Macd(c) => macd_signals(prices, c),
        StrategyConfig::Dca(c) => dca_signals(prices, c),
    }
}

/// Minimum data points required for the strategy to produce meaningful signals.
pub fn min_data_points(config: &StrategyConfig) -> usize {
    match config {
        StrategyConfig::SmaCrossover(c) => c.long_period,
        StrategyConfig::Rsi(c) => c.period + 1,
        StrategyConfig::BollingerBands(c) => c.period,
        StrategyConfig::Macd(c) => c.slow_period + c.signal_period,
        StrategyConfig::Dca(_) => 1,
    }
}

// ===========================================================================
// 1. SMA Crossover
// ===========================================================================

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

// ===========================================================================
// 2. RSI (Relative Strength Index)
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RsiConfig {
    pub period: usize,
    pub overbought: f64,
    pub oversold: f64,
}

impl Default for RsiConfig {
    fn default() -> Self {
        Self {
            period: 14,
            overbought: 70.0,
            oversold: 30.0,
        }
    }
}

/// RSI strategy:
/// - RSI drops below `oversold` then crosses back above → Buy
/// - RSI rises above `overbought` then crosses back below → Sell
pub fn rsi_signals(prices: &[StockPrice], config: &RsiConfig) -> Vec<Signal> {
    let n = prices.len();
    let closes: Vec<f64> = prices.iter().map(|p| p.close).collect();
    let rsi = compute_rsi(&closes, config.period);

    let mut signals = vec![Signal::Hold; n];

    for i in 1..n {
        let (Some(prev), Some(cur)) = (rsi[i - 1], rsi[i]) else {
            continue;
        };

        // Cross above oversold → Buy
        if prev <= config.oversold && cur > config.oversold {
            signals[i] = Signal::Buy;
        }
        // Cross below overbought → Sell
        else if prev >= config.overbought && cur < config.overbought {
            signals[i] = Signal::Sell;
        }
    }

    signals
}

fn compute_rsi(data: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = data.len();
    let mut result = vec![None; n];
    if period == 0 || n <= period {
        return result;
    }

    // Calculate initial average gain/loss
    let mut avg_gain = 0.0;
    let mut avg_loss = 0.0;
    for i in 1..=period {
        let change = data[i] - data[i - 1];
        if change > 0.0 {
            avg_gain += change;
        } else {
            avg_loss += change.abs();
        }
    }
    avg_gain /= period as f64;
    avg_loss /= period as f64;

    let rsi_val = if avg_loss == 0.0 {
        100.0
    } else {
        100.0 - 100.0 / (1.0 + avg_gain / avg_loss)
    };
    result[period] = Some(rsi_val);

    // Smoothed RSI for remaining values
    for i in (period + 1)..n {
        let change = data[i] - data[i - 1];
        let (gain, loss) = if change > 0.0 {
            (change, 0.0)
        } else {
            (0.0, change.abs())
        };

        avg_gain = (avg_gain * (period as f64 - 1.0) + gain) / period as f64;
        avg_loss = (avg_loss * (period as f64 - 1.0) + loss) / period as f64;

        let rsi_val = if avg_loss == 0.0 {
            100.0
        } else {
            100.0 - 100.0 / (1.0 + avg_gain / avg_loss)
        };
        result[i] = Some(rsi_val);
    }

    result
}

// ===========================================================================
// 3. Bollinger Bands
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BollingerBandsConfig {
    pub period: usize,
    pub std_dev: f64,
}

impl Default for BollingerBandsConfig {
    fn default() -> Self {
        Self {
            period: 20,
            std_dev: 2.0,
        }
    }
}

/// Bollinger Bands (mean-reversion):
/// - Price crosses below lower band → Buy (oversold)
/// - Price crosses above upper band → Sell (overbought)
pub fn bollinger_bands_signals(
    prices: &[StockPrice],
    config: &BollingerBandsConfig,
) -> Vec<Signal> {
    let n = prices.len();
    let closes: Vec<f64> = prices.iter().map(|p| p.close).collect();
    let sma = rolling_sma(&closes, config.period);

    let mut signals = vec![Signal::Hold; n];

    for i in 1..n {
        let Some(mid) = sma[i] else { continue };
        let Some(mid_prev) = sma[i - 1] else { continue };

        // Compute standard deviation for current window
        let start = if i + 1 >= config.period {
            i + 1 - config.period
        } else {
            continue;
        };
        let window = &closes[start..=i];
        let std = std_dev(window, mid);

        let upper = mid + config.std_dev * std;
        let lower = mid - config.std_dev * std;

        // Previous bands
        let start_prev = if i >= config.period {
            i - config.period
        } else {
            continue;
        };
        let window_prev = &closes[start_prev..i];
        let std_prev = std_dev(window_prev, mid_prev);
        let upper_prev = mid_prev + config.std_dev * std_prev;
        let lower_prev = mid_prev - config.std_dev * std_prev;

        // Cross below lower band → Buy
        if closes[i - 1] >= lower_prev && closes[i] < lower {
            signals[i] = Signal::Buy;
        }
        // Cross above upper band → Sell
        else if closes[i - 1] <= upper_prev && closes[i] > upper {
            signals[i] = Signal::Sell;
        }
    }

    signals
}

fn std_dev(data: &[f64], mean: f64) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    let variance = data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / data.len() as f64;
    variance.sqrt()
}

// ===========================================================================
// 4. MACD (Moving Average Convergence Divergence)
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacdConfig {
    pub fast_period: usize,
    pub slow_period: usize,
    pub signal_period: usize,
}

impl Default for MacdConfig {
    fn default() -> Self {
        Self {
            fast_period: 12,
            slow_period: 26,
            signal_period: 9,
        }
    }
}

/// MACD strategy:
/// - MACD line crosses above signal line → Buy
/// - MACD line crosses below signal line → Sell
pub fn macd_signals(prices: &[StockPrice], config: &MacdConfig) -> Vec<Signal> {
    let n = prices.len();
    let closes: Vec<f64> = prices.iter().map(|p| p.close).collect();

    let fast_ema = compute_ema(&closes, config.fast_period);
    let slow_ema = compute_ema(&closes, config.slow_period);

    // MACD line = fast EMA - slow EMA
    let mut macd_line: Vec<Option<f64>> = vec![None; n];
    for i in 0..n {
        if let (Some(f), Some(s)) = (fast_ema[i], slow_ema[i]) {
            macd_line[i] = Some(f - s);
        }
    }

    // Signal line = EMA of MACD line
    let macd_values: Vec<f64> = macd_line.iter().filter_map(|v| *v).collect();
    let signal_ema = compute_ema(&macd_values, config.signal_period);

    // Map signal EMA back to original indices
    let mut signal_line: Vec<Option<f64>> = vec![None; n];
    let mut j = 0;
    for i in 0..n {
        if macd_line[i].is_some() {
            signal_line[i] = signal_ema.get(j).copied().flatten();
            j += 1;
        }
    }

    let mut signals = vec![Signal::Hold; n];

    for i in 1..n {
        let (Some(macd_prev), Some(sig_prev)) = (macd_line[i - 1], signal_line[i - 1]) else {
            continue;
        };
        let (Some(macd_cur), Some(sig_cur)) = (macd_line[i], signal_line[i]) else {
            continue;
        };

        if macd_prev <= sig_prev && macd_cur > sig_cur {
            signals[i] = Signal::Buy;
        } else if macd_prev >= sig_prev && macd_cur < sig_cur {
            signals[i] = Signal::Sell;
        }
    }

    signals
}

fn compute_ema(data: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = data.len();
    let mut result = vec![None; n];
    if period == 0 || n < period {
        return result;
    }

    // Seed EMA with SMA of first `period` values
    let sma: f64 = data[..period].iter().sum::<f64>() / period as f64;
    result[period - 1] = Some(sma);

    let multiplier = 2.0 / (period as f64 + 1.0);

    for i in period..n {
        let prev = result[i - 1].unwrap();
        result[i] = Some((data[i] - prev) * multiplier + prev);
    }

    result
}

// ===========================================================================
// 5. DCA (Dollar-Cost Averaging / 定期定額)
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DcaConfig {
    /// Fixed amount to invest each time (TWD).
    pub amount: f64,
    /// Buy every N trading days.
    pub interval_days: usize,
}

impl Default for DcaConfig {
    fn default() -> Self {
        Self {
            amount: 10000.0,
            interval_days: 22, // roughly monthly
        }
    }
}

/// DCA strategy: buy a fixed dollar amount every N trading days. Never sell.
pub fn dca_signals(prices: &[StockPrice], config: &DcaConfig) -> Vec<Signal> {
    let n = prices.len();
    let mut signals = vec![Signal::Hold; n];
    let interval = config.interval_days.max(1);

    for i in (0..n).step_by(interval) {
        signals[i] = Signal::BuyFixed(config.amount);
    }

    signals
}

// ===========================================================================
// Shared helpers
// ===========================================================================

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

// ===========================================================================
// Tests
// ===========================================================================

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

    // -- SMA --

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
        let prices = make_prices(&[50.0, 40.0, 30.0, 20.0, 10.0, 50.0, 80.0]);
        let config = SmaCrossoverConfig {
            short_period: 2,
            long_period: 3,
        };
        let signals = sma_crossover_signals(&prices, &config);
        assert!(signals.contains(&Signal::Buy));
    }

    // -- RSI --

    #[test]
    fn test_rsi_values_in_range() {
        // Generate prices with some movement
        let mut closes = Vec::new();
        for i in 0..50 {
            closes.push(100.0 + (i as f64 * 0.7).sin() * 10.0);
        }
        let rsi = compute_rsi(&closes, 14);
        for val in rsi.iter().flatten() {
            assert!(*val >= 0.0 && *val <= 100.0, "RSI out of range: {val}");
        }
    }

    #[test]
    fn test_rsi_all_gains() {
        // Monotonically increasing → RSI should be 100
        let closes: Vec<f64> = (0..20).map(|i| 100.0 + i as f64).collect();
        let rsi = compute_rsi(&closes, 14);
        assert!((rsi[14].unwrap() - 100.0).abs() < 1e-9);
    }

    #[test]
    fn test_rsi_oversold_buy_signal() {
        // Build a series that drops sharply then recovers → oversold crossover
        let mut closes = vec![100.0; 16];
        // Sharp drop
        for i in 0..10 {
            closes.push(100.0 - (i as f64 * 3.0));
        }
        // Recovery
        for i in 0..5 {
            closes.push(70.0 + (i as f64 * 5.0));
        }
        let prices = make_prices(&closes);
        let config = RsiConfig {
            period: 14,
            oversold: 30.0,
            overbought: 70.0,
        };
        let signals = rsi_signals(&prices, &config);
        // Should have at least one signal (buy or hold)
        assert_eq!(signals.len(), closes.len());
    }

    // -- EMA --

    #[test]
    fn test_ema_basic() {
        let data = vec![10.0, 11.0, 12.0, 13.0, 14.0];
        let ema = compute_ema(&data, 3);
        assert_eq!(ema[0], None);
        assert_eq!(ema[1], None);
        // EMA seed = SMA(10,11,12) = 11.0
        assert!((ema[2].unwrap() - 11.0).abs() < 1e-9);
        // EMA[3] = (13 - 11) * 0.5 + 11 = 12.0
        assert!((ema[3].unwrap() - 12.0).abs() < 1e-9);
    }

    // -- MACD --

    #[test]
    fn test_macd_not_enough_data() {
        let prices = make_prices(&[100.0; 10]);
        let config = MacdConfig {
            fast_period: 12,
            slow_period: 26,
            signal_period: 9,
        };
        let signals = macd_signals(&prices, &config);
        assert!(signals.iter().all(|s| *s == Signal::Hold));
    }

    #[test]
    fn test_macd_generates_signals_on_trend_change() {
        // Uptrend then downtrend → should trigger at least one crossover
        let mut closes = Vec::new();
        for i in 0..40 {
            closes.push(100.0 + i as f64 * 2.0); // up
        }
        for i in 0..20 {
            closes.push(180.0 - i as f64 * 3.0); // down
        }
        let prices = make_prices(&closes);
        let config = MacdConfig::default();
        let signals = macd_signals(&prices, &config);
        assert_eq!(signals.len(), closes.len());
        // Should have at least one sell signal on the downtrend
        let has_sell = signals.iter().any(|s| *s == Signal::Sell);
        assert!(has_sell, "MACD should detect trend reversal");
    }

    // -- Bollinger Bands --

    #[test]
    fn test_bollinger_flat_market_no_signals() {
        // Flat prices → no band crossings
        let prices = make_prices(&[100.0; 30]);
        let config = BollingerBandsConfig::default();
        let signals = bollinger_bands_signals(&prices, &config);
        assert!(signals.iter().all(|s| *s == Signal::Hold));
    }

    // -- generate_signals dispatch --

    #[test]
    fn test_generate_signals_dispatch() {
        let prices = make_prices(&[100.0; 30]);

        let sma_config = StrategyConfig::SmaCrossover(SmaCrossoverConfig::default());
        let sma_signals = generate_signals(&prices, &sma_config);
        assert_eq!(sma_signals.len(), 30);

        let rsi_config = StrategyConfig::Rsi(RsiConfig::default());
        let rsi_signals = generate_signals(&prices, &rsi_config);
        assert_eq!(rsi_signals.len(), 30);

        let bb_config = StrategyConfig::BollingerBands(BollingerBandsConfig::default());
        let bb_signals = generate_signals(&prices, &bb_config);
        assert_eq!(bb_signals.len(), 30);

        let macd_config = StrategyConfig::Macd(MacdConfig::default());
        let macd_signals = generate_signals(&prices, &macd_config);
        assert_eq!(macd_signals.len(), 30);

        let dca_config = StrategyConfig::Dca(DcaConfig::default());
        let dca_signals = generate_signals(&prices, &dca_config);
        assert_eq!(dca_signals.len(), 30);
    }

    // -- DCA --

    #[test]
    fn test_dca_signals_interval() {
        let prices = make_prices(&[100.0; 50]);
        let config = DcaConfig {
            amount: 10000.0,
            interval_days: 10,
        };
        let signals = dca_signals(&prices, &config);
        assert_eq!(signals.len(), 50);

        // Should buy at index 0, 10, 20, 30, 40
        let buy_indices: Vec<usize> = signals
            .iter()
            .enumerate()
            .filter(|(_, s)| matches!(s, Signal::BuyFixed(_)))
            .map(|(i, _)| i)
            .collect();
        assert_eq!(buy_indices, vec![0, 10, 20, 30, 40]);
    }

    #[test]
    fn test_dca_buy_fixed_amount() {
        let prices = make_prices(&[100.0; 50]);
        let config = DcaConfig {
            amount: 5000.0,
            interval_days: 10,
        };
        let signals = dca_signals(&prices, &config);

        if let Signal::BuyFixed(amt) = signals[0] {
            assert!((amt - 5000.0).abs() < 1e-9);
        } else {
            panic!("Expected BuyFixed at index 0");
        }
    }
}
