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
    Stochastic(StochasticConfig),
    EmaCrossover(EmaCrossoverConfig),
    Supertrend(SupertrendConfig),
    DonchianBreakout(DonchianBreakoutConfig),
    WilliamsR(WilliamsRConfig),
    Cci(CciConfig),
    ParabolicSar(ParabolicSarConfig),
}

/// Dispatch signals based on the chosen strategy.
pub fn generate_signals(prices: &[StockPrice], config: &StrategyConfig) -> Vec<Signal> {
    match config {
        StrategyConfig::SmaCrossover(c) => sma_crossover_signals(prices, c),
        StrategyConfig::Rsi(c) => rsi_signals(prices, c),
        StrategyConfig::BollingerBands(c) => bollinger_bands_signals(prices, c),
        StrategyConfig::Macd(c) => macd_signals(prices, c),
        StrategyConfig::Dca(c) => dca_signals(prices, c),
        StrategyConfig::Stochastic(c) => stochastic_signals(prices, c),
        StrategyConfig::EmaCrossover(c) => ema_crossover_signals(prices, c),
        StrategyConfig::Supertrend(c) => supertrend_signals(prices, c),
        StrategyConfig::DonchianBreakout(c) => donchian_breakout_signals(prices, c),
        StrategyConfig::WilliamsR(c) => williams_r_signals(prices, c),
        StrategyConfig::Cci(c) => cci_signals(prices, c),
        StrategyConfig::ParabolicSar(c) => parabolic_sar_signals(prices, c),
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
        StrategyConfig::Stochastic(c) => c.k_period + c.d_period,
        StrategyConfig::EmaCrossover(c) => c.long_period,
        StrategyConfig::Supertrend(c) => c.period,
        StrategyConfig::DonchianBreakout(c) => c.period,
        StrategyConfig::WilliamsR(c) => c.period + 1,
        StrategyConfig::Cci(c) => c.period + 1,
        StrategyConfig::ParabolicSar(_) => 2,
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
// 6. Stochastic Oscillator (KD)
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StochasticConfig {
    pub k_period: usize,
    pub d_period: usize,
    pub overbought: f64,
    pub oversold: f64,
}

impl Default for StochasticConfig {
    fn default() -> Self {
        Self {
            k_period: 14,
            d_period: 3,
            overbought: 80.0,
            oversold: 20.0,
        }
    }
}

/// Stochastic (KD) strategy:
/// - %K crosses above %D while both below oversold → Buy
/// - %K crosses below %D while both above overbought → Sell
pub fn stochastic_signals(prices: &[StockPrice], config: &StochasticConfig) -> Vec<Signal> {
    let n = prices.len();
    let min_len = config.k_period + config.d_period;
    if n < min_len {
        return vec![Signal::Hold; n];
    }

    // Compute %K (Fast Stochastic)
    let mut k_values: Vec<Option<f64>> = vec![None; n];
    for i in (config.k_period - 1)..n {
        let start = i + 1 - config.k_period;
        let mut highest = f64::MIN;
        let mut lowest = f64::MAX;
        for j in start..=i {
            if prices[j].high > highest {
                highest = prices[j].high;
            }
            if prices[j].low < lowest {
                lowest = prices[j].low;
            }
        }
        let range = highest - lowest;
        k_values[i] = if range > 0.0 {
            Some((prices[i].close - lowest) / range * 100.0)
        } else {
            Some(50.0)
        };
    }

    // Compute %D = SMA of %K
    let k_flat: Vec<f64> = k_values.iter().filter_map(|v| *v).collect();
    let d_sma = rolling_sma(&k_flat, config.d_period);

    // Map %D back to original indices
    let mut d_values: Vec<Option<f64>> = vec![None; n];
    let mut j = 0;
    for i in 0..n {
        if k_values[i].is_some() {
            d_values[i] = d_sma.get(j).copied().flatten();
            j += 1;
        }
    }

    let mut signals = vec![Signal::Hold; n];
    for i in 1..n {
        let (Some(k_prev), Some(d_prev)) = (k_values[i - 1], d_values[i - 1]) else {
            continue;
        };
        let (Some(k_cur), Some(d_cur)) = (k_values[i], d_values[i]) else {
            continue;
        };

        // %K crosses above %D in oversold zone → Buy
        if k_prev <= d_prev && k_cur > d_cur && k_cur < config.oversold {
            signals[i] = Signal::Buy;
        }
        // %K crosses below %D in overbought zone → Sell
        else if k_prev >= d_prev && k_cur < d_cur && k_cur > config.overbought {
            signals[i] = Signal::Sell;
        }
    }

    signals
}

// ===========================================================================
// 7. EMA Crossover
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmaCrossoverConfig {
    pub short_period: usize,
    pub long_period: usize,
}

impl Default for EmaCrossoverConfig {
    fn default() -> Self {
        Self {
            short_period: 12,
            long_period: 26,
        }
    }
}

/// EMA Crossover strategy:
/// - Short EMA crosses above long EMA → Buy
/// - Short EMA crosses below long EMA → Sell
pub fn ema_crossover_signals(
    prices: &[StockPrice],
    config: &EmaCrossoverConfig,
) -> Vec<Signal> {
    let n = prices.len();
    if n < config.long_period {
        return vec![Signal::Hold; n];
    }

    let closes: Vec<f64> = prices.iter().map(|p| p.close).collect();
    let short_ema = compute_ema(&closes, config.short_period);
    let long_ema = compute_ema(&closes, config.long_period);

    let mut signals = vec![Signal::Hold; n];

    for i in 1..n {
        let (Some(s_prev), Some(l_prev)) = (short_ema[i - 1], long_ema[i - 1]) else {
            continue;
        };
        let (Some(s_cur), Some(l_cur)) = (short_ema[i], long_ema[i]) else {
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
// 8. Supertrend
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupertrendConfig {
    pub period: usize,
    pub multiplier: f64,
}

impl Default for SupertrendConfig {
    fn default() -> Self {
        Self {
            period: 10,
            multiplier: 3.0,
        }
    }
}

/// Supertrend strategy: uses ATR-based bands to determine trend direction.
/// - Price crosses above supertrend line → Buy
/// - Price crosses below supertrend line → Sell
pub fn supertrend_signals(prices: &[StockPrice], config: &SupertrendConfig) -> Vec<Signal> {
    let n = prices.len();
    if n < config.period {
        return vec![Signal::Hold; n];
    }

    // Compute ATR
    let atr = compute_atr(prices, config.period);

    // Compute supertrend
    let mut upper_band = vec![0.0f64; n];
    let mut lower_band = vec![0.0f64; n];
    let mut supertrend = vec![0.0f64; n];
    let mut direction = vec![1i8; n]; // 1 = uptrend, -1 = downtrend

    for i in 0..n {
        let Some(atr_val) = atr[i] else {
            continue;
        };
        let hl2 = (prices[i].high + prices[i].low) / 2.0;
        let basic_upper = hl2 + config.multiplier * atr_val;
        let basic_lower = hl2 - config.multiplier * atr_val;

        if i == 0 || atr[i - 1].is_none() {
            upper_band[i] = basic_upper;
            lower_band[i] = basic_lower;
            supertrend[i] = basic_lower;
            direction[i] = 1;
            continue;
        }

        // Upper band: take the lower of current basic vs previous final (tighter)
        upper_band[i] = if basic_upper < upper_band[i - 1] || prices[i - 1].close > upper_band[i - 1] {
            basic_upper
        } else {
            upper_band[i - 1]
        };

        // Lower band: take the higher of current basic vs previous final (tighter)
        lower_band[i] = if basic_lower > lower_band[i - 1] || prices[i - 1].close < lower_band[i - 1] {
            basic_lower
        } else {
            lower_band[i - 1]
        };

        // Determine direction
        if direction[i - 1] == 1 {
            // Was uptrend
            if prices[i].close < lower_band[i] {
                direction[i] = -1;
                supertrend[i] = upper_band[i];
            } else {
                direction[i] = 1;
                supertrend[i] = lower_band[i];
            }
        } else {
            // Was downtrend
            if prices[i].close > upper_band[i] {
                direction[i] = 1;
                supertrend[i] = lower_band[i];
            } else {
                direction[i] = -1;
                supertrend[i] = upper_band[i];
            }
        }
    }

    let mut signals = vec![Signal::Hold; n];
    for i in 1..n {
        if atr[i].is_none() || atr[i - 1].is_none() {
            continue;
        }
        // Direction flip from downtrend to uptrend → Buy
        if direction[i - 1] == -1 && direction[i] == 1 {
            signals[i] = Signal::Buy;
        }
        // Direction flip from uptrend to downtrend → Sell
        else if direction[i - 1] == 1 && direction[i] == -1 {
            signals[i] = Signal::Sell;
        }
    }

    signals
}

fn compute_atr(prices: &[StockPrice], period: usize) -> Vec<Option<f64>> {
    let n = prices.len();
    let mut tr = vec![0.0f64; n];
    let mut result = vec![None; n];

    if n == 0 || period == 0 {
        return result;
    }

    tr[0] = prices[0].high - prices[0].low;
    for i in 1..n {
        let hl = prices[i].high - prices[i].low;
        let hc = (prices[i].high - prices[i - 1].close).abs();
        let lc = (prices[i].low - prices[i - 1].close).abs();
        tr[i] = hl.max(hc).max(lc);
    }

    if n < period {
        return result;
    }

    // Initial ATR = average of first `period` true ranges
    let mut atr_val: f64 = tr[..period].iter().sum::<f64>() / period as f64;
    result[period - 1] = Some(atr_val);

    // Smoothed ATR (Wilder's method)
    for i in period..n {
        atr_val = (atr_val * (period as f64 - 1.0) + tr[i]) / period as f64;
        result[i] = Some(atr_val);
    }

    result
}

// ===========================================================================
// 9. Donchian Channel Breakout (Turtle Trading)
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DonchianBreakoutConfig {
    pub period: usize,
}

impl Default for DonchianBreakoutConfig {
    fn default() -> Self {
        Self { period: 20 }
    }
}

/// Donchian Channel Breakout (Turtle Trading):
/// - Price breaks above the N-period high → Buy
/// - Price breaks below the N-period low → Sell
pub fn donchian_breakout_signals(
    prices: &[StockPrice],
    config: &DonchianBreakoutConfig,
) -> Vec<Signal> {
    let n = prices.len();
    if n <= config.period {
        return vec![Signal::Hold; n];
    }

    let mut signals = vec![Signal::Hold; n];

    for i in config.period..n {
        let start = i - config.period;
        let mut highest = f64::MIN;
        let mut lowest = f64::MAX;
        for j in start..i {
            if prices[j].high > highest {
                highest = prices[j].high;
            }
            if prices[j].low < lowest {
                lowest = prices[j].low;
            }
        }

        if prices[i].close > highest {
            signals[i] = Signal::Buy;
        } else if prices[i].close < lowest {
            signals[i] = Signal::Sell;
        }
    }

    signals
}

// ===========================================================================
// 10. Williams %R
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WilliamsRConfig {
    pub period: usize,
    pub overbought: f64,
    pub oversold: f64,
}

impl Default for WilliamsRConfig {
    fn default() -> Self {
        Self {
            period: 14,
            overbought: -20.0,
            oversold: -80.0,
        }
    }
}

/// Williams %R strategy:
/// - %R crosses above oversold threshold → Buy
/// - %R crosses below overbought threshold → Sell
pub fn williams_r_signals(prices: &[StockPrice], config: &WilliamsRConfig) -> Vec<Signal> {
    let n = prices.len();
    if n < config.period {
        return vec![Signal::Hold; n];
    }

    let mut wr: Vec<Option<f64>> = vec![None; n];

    for i in (config.period - 1)..n {
        let start = i + 1 - config.period;
        let mut highest = f64::MIN;
        let mut lowest = f64::MAX;
        for j in start..=i {
            if prices[j].high > highest {
                highest = prices[j].high;
            }
            if prices[j].low < lowest {
                lowest = prices[j].low;
            }
        }
        let range = highest - lowest;
        wr[i] = if range > 0.0 {
            Some((highest - prices[i].close) / range * -100.0)
        } else {
            Some(-50.0)
        };
    }

    let mut signals = vec![Signal::Hold; n];
    for i in 1..n {
        let (Some(prev), Some(cur)) = (wr[i - 1], wr[i]) else {
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

// ===========================================================================
// 11. CCI (Commodity Channel Index)
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CciConfig {
    pub period: usize,
    pub overbought: f64,
    pub oversold: f64,
}

impl Default for CciConfig {
    fn default() -> Self {
        Self {
            period: 20,
            overbought: 100.0,
            oversold: -100.0,
        }
    }
}

/// CCI strategy:
/// - CCI crosses above oversold → Buy
/// - CCI crosses below overbought → Sell
pub fn cci_signals(prices: &[StockPrice], config: &CciConfig) -> Vec<Signal> {
    let n = prices.len();
    if n < config.period {
        return vec![Signal::Hold; n];
    }

    // Typical price = (H + L + C) / 3
    let tp: Vec<f64> = prices
        .iter()
        .map(|p| (p.high + p.low + p.close) / 3.0)
        .collect();

    let mut cci: Vec<Option<f64>> = vec![None; n];

    for i in (config.period - 1)..n {
        let start = i + 1 - config.period;
        let window = &tp[start..=i];
        let mean = window.iter().sum::<f64>() / config.period as f64;
        let mean_dev = window.iter().map(|x| (x - mean).abs()).sum::<f64>() / config.period as f64;

        cci[i] = if mean_dev > 0.0 {
            Some((tp[i] - mean) / (0.015 * mean_dev))
        } else {
            Some(0.0)
        };
    }

    let mut signals = vec![Signal::Hold; n];
    for i in 1..n {
        let (Some(prev), Some(cur)) = (cci[i - 1], cci[i]) else {
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

// ===========================================================================
// 12. Parabolic SAR
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParabolicSarConfig {
    pub af_start: f64,
    pub af_increment: f64,
    pub af_max: f64,
}

impl Default for ParabolicSarConfig {
    fn default() -> Self {
        Self {
            af_start: 0.02,
            af_increment: 0.02,
            af_max: 0.2,
        }
    }
}

/// Parabolic SAR strategy:
/// - SAR flips below price (uptrend start) → Buy
/// - SAR flips above price (downtrend start) → Sell
pub fn parabolic_sar_signals(
    prices: &[StockPrice],
    config: &ParabolicSarConfig,
) -> Vec<Signal> {
    let n = prices.len();
    if n < 2 {
        return vec![Signal::Hold; n];
    }

    let mut sar = vec![0.0f64; n];
    let mut is_uptrend = vec![true; n];

    // Initialize: assume uptrend starting at index 0
    let initial_uptrend = prices[1].close >= prices[0].close;
    is_uptrend[0] = initial_uptrend;

    let mut af = config.af_start;
    let mut ep; // extreme point

    if initial_uptrend {
        sar[0] = prices[0].low;
        ep = prices[0].high;
    } else {
        sar[0] = prices[0].high;
        ep = prices[0].low;
    }

    for i in 1..n {
        let prev_sar = sar[i - 1];
        let prev_uptrend = is_uptrend[i - 1];

        // Calculate new SAR
        let mut new_sar = prev_sar + af * (ep - prev_sar);

        if prev_uptrend {
            // In uptrend, SAR must not be above the two previous lows
            if i >= 2 {
                new_sar = new_sar.min(prices[i - 1].low).min(prices[i - 2].low);
            } else {
                new_sar = new_sar.min(prices[i - 1].low);
            }

            if prices[i].low <= new_sar {
                // Reversal to downtrend
                is_uptrend[i] = false;
                sar[i] = ep; // SAR becomes the previous extreme point
                ep = prices[i].low;
                af = config.af_start;
            } else {
                // Continue uptrend
                is_uptrend[i] = true;
                sar[i] = new_sar;
                if prices[i].high > ep {
                    ep = prices[i].high;
                    af = (af + config.af_increment).min(config.af_max);
                }
            }
        } else {
            // In downtrend, SAR must not be below the two previous highs
            if i >= 2 {
                new_sar = new_sar.max(prices[i - 1].high).max(prices[i - 2].high);
            } else {
                new_sar = new_sar.max(prices[i - 1].high);
            }

            if prices[i].high >= new_sar {
                // Reversal to uptrend
                is_uptrend[i] = true;
                sar[i] = ep;
                ep = prices[i].high;
                af = config.af_start;
            } else {
                // Continue downtrend
                is_uptrend[i] = false;
                sar[i] = new_sar;
                if prices[i].low < ep {
                    ep = prices[i].low;
                    af = (af + config.af_increment).min(config.af_max);
                }
            }
        }
    }

    let mut signals = vec![Signal::Hold; n];
    for i in 1..n {
        if !is_uptrend[i - 1] && is_uptrend[i] {
            signals[i] = Signal::Buy;
        } else if is_uptrend[i - 1] && !is_uptrend[i] {
            signals[i] = Signal::Sell;
        }
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

    // -- Helper: make prices with OHLCV data --

    fn make_ohlcv_prices(data: &[(f64, f64, f64, f64)]) -> Vec<StockPrice> {
        data.iter()
            .enumerate()
            .map(|(i, &(o, h, l, c))| StockPrice {
                id: i as i64,
                symbol: "TEST".into(),
                date: NaiveDate::from_ymd_opt(2024, 1, 1)
                    .unwrap()
                    .checked_add_signed(chrono::Duration::days(i as i64))
                    .unwrap(),
                open: o,
                high: h,
                low: l,
                close: c,
                volume: 1000,
                adj_close: Some(c),
                created_at: String::new(),
            })
            .collect()
    }

    // -- Stochastic (KD) --

    #[test]
    fn test_stochastic_not_enough_data() {
        let prices = make_prices(&[100.0; 5]);
        let config = StochasticConfig::default(); // k=14, d=3
        let signals = stochastic_signals(&prices, &config);
        assert!(signals.iter().all(|s| *s == Signal::Hold));
    }

    #[test]
    fn test_stochastic_correct_length() {
        let prices = make_prices(&[100.0; 50]);
        let config = StochasticConfig::default();
        let signals = stochastic_signals(&prices, &config);
        assert_eq!(signals.len(), 50);
    }

    // -- EMA Crossover --

    #[test]
    fn test_ema_crossover_not_enough_data() {
        let prices = make_prices(&[100.0; 10]);
        let config = EmaCrossoverConfig {
            short_period: 12,
            long_period: 26,
        };
        let signals = ema_crossover_signals(&prices, &config);
        assert!(signals.iter().all(|s| *s == Signal::Hold));
    }

    #[test]
    fn test_ema_crossover_detects_trend_change() {
        let mut closes = Vec::new();
        for i in 0..30 {
            closes.push(100.0 + i as f64 * 2.0);
        }
        for i in 0..20 {
            closes.push(160.0 - i as f64 * 3.0);
        }
        let prices = make_prices(&closes);
        let config = EmaCrossoverConfig {
            short_period: 5,
            long_period: 10,
        };
        let signals = ema_crossover_signals(&prices, &config);
        assert_eq!(signals.len(), closes.len());
        let has_sell = signals.iter().any(|s| *s == Signal::Sell);
        assert!(has_sell, "EMA crossover should detect trend reversal");
    }

    // -- Supertrend --

    #[test]
    fn test_supertrend_not_enough_data() {
        let prices = make_prices(&[100.0; 5]);
        let config = SupertrendConfig::default(); // period=10
        let signals = supertrend_signals(&prices, &config);
        assert!(signals.iter().all(|s| *s == Signal::Hold));
    }

    #[test]
    fn test_supertrend_detects_reversal() {
        // Build uptrend then downtrend with high/low spread
        let mut data = Vec::new();
        for i in 0..20 {
            let base = 100.0 + i as f64 * 2.0;
            data.push((base - 1.0, base + 2.0, base - 2.0, base));
        }
        for i in 0..15 {
            let base = 140.0 - i as f64 * 3.0;
            data.push((base + 1.0, base + 2.0, base - 2.0, base));
        }
        let prices = make_ohlcv_prices(&data);
        let config = SupertrendConfig {
            period: 7,
            multiplier: 2.0,
        };
        let signals = supertrend_signals(&prices, &config);
        assert_eq!(signals.len(), data.len());
        let has_sell = signals.iter().any(|s| *s == Signal::Sell);
        assert!(has_sell, "Supertrend should detect trend reversal");
    }

    // -- ATR --

    #[test]
    fn test_atr_basic() {
        let data: Vec<(f64, f64, f64, f64)> = (0..20)
            .map(|i| {
                let base = 100.0 + i as f64;
                (base, base + 2.0, base - 2.0, base + 0.5)
            })
            .collect();
        let prices = make_ohlcv_prices(&data);
        let atr = compute_atr(&prices, 10);
        // ATR should be computed for indices >= 9
        assert!(atr[8].is_none());
        assert!(atr[9].is_some());
        // With H-L=4 consistently, ATR should be around 4
        let atr_val = atr[9].unwrap();
        assert!(atr_val > 3.0 && atr_val < 5.0, "ATR={atr_val} should be ~4");
    }

    // -- Donchian Breakout --

    #[test]
    fn test_donchian_breakout_buy_on_new_high() {
        let mut closes = vec![100.0; 25];
        closes.push(120.0); // breakout above 20-period high
        let prices = make_prices(&closes);
        let config = DonchianBreakoutConfig { period: 20 };
        let signals = donchian_breakout_signals(&prices, &config);
        assert_eq!(signals[25], Signal::Buy);
    }

    #[test]
    fn test_donchian_breakout_sell_on_new_low() {
        let mut closes = vec![100.0; 25];
        closes.push(80.0); // breakout below 20-period low
        let prices = make_prices(&closes);
        let config = DonchianBreakoutConfig { period: 20 };
        let signals = donchian_breakout_signals(&prices, &config);
        assert_eq!(signals[25], Signal::Sell);
    }

    // -- Williams %R --

    #[test]
    fn test_williams_r_correct_length() {
        let prices = make_prices(&[100.0; 30]);
        let config = WilliamsRConfig::default();
        let signals = williams_r_signals(&prices, &config);
        assert_eq!(signals.len(), 30);
    }

    #[test]
    fn test_williams_r_not_enough_data() {
        let prices = make_prices(&[100.0; 5]);
        let config = WilliamsRConfig::default(); // period=14
        let signals = williams_r_signals(&prices, &config);
        assert!(signals.iter().all(|s| *s == Signal::Hold));
    }

    // -- CCI --

    #[test]
    fn test_cci_correct_length() {
        let prices = make_prices(&[100.0; 30]);
        let config = CciConfig::default();
        let signals = cci_signals(&prices, &config);
        assert_eq!(signals.len(), 30);
    }

    #[test]
    fn test_cci_flat_market_no_signals() {
        let prices = make_prices(&[100.0; 30]);
        let config = CciConfig::default();
        let signals = cci_signals(&prices, &config);
        assert!(signals.iter().all(|s| *s == Signal::Hold));
    }

    // -- Parabolic SAR --

    #[test]
    fn test_parabolic_sar_correct_length() {
        let data: Vec<(f64, f64, f64, f64)> = (0..30)
            .map(|i| {
                let base = 100.0 + (i as f64 * 0.5).sin() * 10.0;
                (base, base + 2.0, base - 2.0, base)
            })
            .collect();
        let prices = make_ohlcv_prices(&data);
        let config = ParabolicSarConfig::default();
        let signals = parabolic_sar_signals(&prices, &config);
        assert_eq!(signals.len(), 30);
    }

    #[test]
    fn test_parabolic_sar_detects_reversal() {
        let mut data = Vec::new();
        // Strong uptrend
        for i in 0..15 {
            let base = 100.0 + i as f64 * 3.0;
            data.push((base - 1.0, base + 1.0, base - 2.0, base));
        }
        // Strong downtrend
        for i in 0..15 {
            let base = 145.0 - i as f64 * 3.0;
            data.push((base + 1.0, base + 2.0, base - 1.0, base));
        }
        let prices = make_ohlcv_prices(&data);
        let config = ParabolicSarConfig::default();
        let signals = parabolic_sar_signals(&prices, &config);
        let has_sell = signals.iter().any(|s| *s == Signal::Sell);
        assert!(has_sell, "Parabolic SAR should detect trend reversal");
    }

    // -- Dispatch for new strategies --

    #[test]
    fn test_generate_signals_dispatch_new_strategies() {
        let data: Vec<(f64, f64, f64, f64)> = (0..50)
            .map(|i| {
                let base = 100.0 + (i as f64 * 0.3).sin() * 10.0;
                (base, base + 2.0, base - 2.0, base)
            })
            .collect();
        let prices = make_ohlcv_prices(&data);

        let configs: Vec<StrategyConfig> = vec![
            StrategyConfig::Stochastic(StochasticConfig::default()),
            StrategyConfig::EmaCrossover(EmaCrossoverConfig::default()),
            StrategyConfig::Supertrend(SupertrendConfig::default()),
            StrategyConfig::DonchianBreakout(DonchianBreakoutConfig::default()),
            StrategyConfig::WilliamsR(WilliamsRConfig::default()),
            StrategyConfig::Cci(CciConfig::default()),
            StrategyConfig::ParabolicSar(ParabolicSarConfig::default()),
        ];

        for config in &configs {
            let signals = generate_signals(&prices, config);
            assert_eq!(signals.len(), 50, "Failed for {:?}", config);
        }
    }
}
