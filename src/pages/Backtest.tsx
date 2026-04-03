import { useState } from "react";
import { useTranslation } from "react-i18next";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { SidebarTrigger } from "@/components/ui/sidebar";
import { ThemeToggle } from "@/components/common/ThemeToggle";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { BacktestChart } from "@/components/charts/BacktestChart";
import { runBacktest } from "@/services/stockService";
import type { StrategyConfig } from "@/services/stockService";
import type { BacktestResult } from "@/types/stock";

const STRATEGY_TYPES = [
  "SmaCrossover",
  "Rsi",
  "BollingerBands",
  "Macd",
  "Dca",
] as const;
type StrategyType = (typeof STRATEGY_TYPES)[number];

const STRATEGY_COLORS: Record<StrategyType, string> = {
  SmaCrossover: "#3b82f6", // blue
  Rsi: "#f59e0b",          // amber
  BollingerBands: "#10b981", // emerald
  Macd: "#8b5cf6",          // violet
  Dca: "#ec4899",           // pink
};

function defaultStrategyConfig(type: StrategyType): StrategyConfig {
  switch (type) {
    case "SmaCrossover":
      return { type: "SmaCrossover", short_period: 5, long_period: 20 };
    case "Rsi":
      return { type: "Rsi", period: 14, overbought: 70, oversold: 30 };
    case "BollingerBands":
      return { type: "BollingerBands", period: 20, std_dev: 2.0 };
    case "Macd":
      return { type: "Macd", fast_period: 12, slow_period: 26, signal_period: 9 };
    case "Dca":
      return { type: "Dca", amount: 10000, interval_days: 22 };
  }
}

function formatNumber(n: number, decimals = 0): string {
  return n.toLocaleString("zh-TW", {
    minimumFractionDigits: decimals,
    maximumFractionDigits: decimals,
  });
}

interface CompareEntry {
  strategy: StrategyType;
  result: BacktestResult;
}

export default function Backtest() {
  const { t } = useTranslation();

  // Form state
  const [symbol, setSymbol] = useState("2330.TW");
  const [startDate, setStartDate] = useState("2023-01-01");
  const [endDate, setEndDate] = useState("2024-12-31");
  const [initialCapital, setInitialCapital] = useState(1000000);
  const [strategyType, setStrategyType] = useState<StrategyType>("SmaCrossover");
  const [strategyConfig, setStrategyConfig] = useState<StrategyConfig>(
    defaultStrategyConfig("SmaCrossover")
  );

  // Single result state
  const [result, setResult] = useState<BacktestResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Compare mode state
  const [compareResults, setCompareResults] = useState<CompareEntry[]>([]);
  const [comparing, setComparing] = useState(false);

  const handleStrategyChange = (type: StrategyType) => {
    setStrategyType(type);
    setStrategyConfig(defaultStrategyConfig(type));
  };

  const updateParam = (key: string, value: number) => {
    setStrategyConfig((prev) => ({ ...prev, [key]: value }));
  };

  const handleRun = async () => {
    setLoading(true);
    setError(null);
    setResult(null);
    setCompareResults([]);
    try {
      const res = await runBacktest({
        symbol,
        start_date: startDate,
        end_date: endDate,
        initial_capital: initialCapital,
        strategy: strategyConfig,
      });
      setResult(res);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const handleCompareAll = async () => {
    setComparing(true);
    setError(null);
    setResult(null);
    setCompareResults([]);

    const entries: CompareEntry[] = [];
    const errors: string[] = [];

    // Run all strategies in parallel
    const promises = STRATEGY_TYPES.map(async (st) => {
      try {
        const res = await runBacktest({
          symbol,
          start_date: startDate,
          end_date: endDate,
          initial_capital: initialCapital,
          strategy: defaultStrategyConfig(st),
        });
        return { strategy: st, result: res } as CompareEntry;
      } catch (e) {
        errors.push(`${t(`backtest.strategies.${st}`)}: ${String(e)}`);
        return null;
      }
    });

    const results = await Promise.all(promises);
    for (const r of results) {
      if (r) entries.push(r);
    }

    setCompareResults(entries);
    if (errors.length > 0) {
      setError(errors.join("\n"));
    }
    setComparing(false);
  };

  return (
    <div className="flex h-screen flex-col">
      {/* Header */}
      <header className="flex h-12 shrink-0 items-center gap-2 border-b px-4">
        <SidebarTrigger className="-ml-1" />
        <Separator orientation="vertical" className="mr-2 h-4" />
        <h1 className="text-sm font-semibold">{t("backtest.title")}</h1>
        <div className="ml-auto">
          <ThemeToggle />
        </div>
      </header>

      <div className="flex-1 overflow-auto p-4 space-y-4">
        {/* Config Panel */}
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-base">{t("backtest.config")}</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            {/* Basic params */}
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
              <div className="space-y-1">
                <label className="text-xs text-muted-foreground">
                  {t("backtest.symbol")}
                </label>
                <Input
                  value={symbol}
                  onChange={(e) => setSymbol(e.target.value)}
                  placeholder="2330.TW"
                />
              </div>
              <div className="space-y-1">
                <label className="text-xs text-muted-foreground">
                  {t("backtest.startDate")}
                </label>
                <Input
                  type="date"
                  value={startDate}
                  onChange={(e) => setStartDate(e.target.value)}
                />
              </div>
              <div className="space-y-1">
                <label className="text-xs text-muted-foreground">
                  {t("backtest.endDate")}
                </label>
                <Input
                  type="date"
                  value={endDate}
                  onChange={(e) => setEndDate(e.target.value)}
                />
              </div>
              <div className="space-y-1">
                <label className="text-xs text-muted-foreground">
                  {t("backtest.initialCapital")}
                </label>
                <Input
                  type="number"
                  value={initialCapital}
                  onChange={(e) => setInitialCapital(Number(e.target.value))}
                />
              </div>
            </div>

            {/* Strategy selection */}
            <div className="space-y-2">
              <label className="text-xs text-muted-foreground">
                {t("backtest.strategy")}
              </label>
              <div className="flex gap-2">
                {STRATEGY_TYPES.map((st) => (
                  <Button
                    key={st}
                    variant={strategyType === st ? "default" : "outline"}
                    size="sm"
                    onClick={() => handleStrategyChange(st)}
                  >
                    {t(`backtest.strategies.${st}`)}
                  </Button>
                ))}
              </div>
            </div>

            {/* Strategy-specific params */}
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
              <StrategyParams
                config={strategyConfig}
                onUpdate={updateParam}
                t={t}
              />
            </div>

            <div className="flex gap-2">
              <Button onClick={handleRun} disabled={loading || comparing || !symbol}>
                {loading ? t("backtest.running") : t("backtest.run")}
              </Button>
              <Button
                variant="outline"
                onClick={handleCompareAll}
                disabled={loading || comparing || !symbol}
              >
                {comparing ? t("backtest.comparing") : t("backtest.compareAll")}
              </Button>
            </div>
            {error && (
              <p className="text-sm text-destructive whitespace-pre-line">{error}</p>
            )}
          </CardContent>
        </Card>

        {/* Compare Results */}
        {compareResults.length > 0 && (
          <>
            {/* Benchmark Banner */}
            <BenchmarkBanner benchmark={compareResults[0].result.benchmark} symbol={symbol} initialCapital={initialCapital} t={t} />

            {/* Comparison Table */}
            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-base">
                  {t("backtest.compareResult")}
                </CardTitle>
              </CardHeader>
              <CardContent>
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>{t("backtest.strategy")}</TableHead>
                      <TableHead className="text-right">{t("backtest.totalReturn")}</TableHead>
                      <TableHead className="text-right">{t("backtest.maxDrawdown")}</TableHead>
                      <TableHead className="text-right">{t("backtest.winRate")}</TableHead>
                      <TableHead className="text-right">{t("backtest.totalInvested")}</TableHead>
                      <TableHead className="text-right">{t("backtest.totalTrades")}</TableHead>
                      <TableHead className="text-right">{t("backtest.finalEquity")}</TableHead>
                      <TableHead className="text-right">{t("backtest.totalCommission")}</TableHead>
                      <TableHead className="text-right">{t("backtest.totalTax")}</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {compareResults.map(({ strategy, result: r }) => (
                      <TableRow key={strategy}>
                        <TableCell>
                          <div className="flex items-center gap-2">
                            <div
                              className="h-3 w-3 rounded-full shrink-0"
                              style={{ backgroundColor: STRATEGY_COLORS[strategy] }}
                            />
                            {t(`backtest.strategies.${strategy}`)}
                          </div>
                        </TableCell>
                        <TableCell
                          className={`text-right font-mono font-semibold ${
                            r.metrics.total_return_pct >= 0
                              ? "text-green-600 dark:text-green-400"
                              : "text-red-600 dark:text-red-400"
                          }`}
                        >
                          {r.metrics.total_return_pct >= 0 ? "+" : ""}
                          {r.metrics.total_return_pct.toFixed(2)}%
                        </TableCell>
                        <TableCell className="text-right font-mono text-red-600 dark:text-red-400">
                          -{r.metrics.max_drawdown_pct.toFixed(2)}%
                        </TableCell>
                        <TableCell className="text-right font-mono">
                          {r.metrics.win_rate_pct.toFixed(1)}%
                        </TableCell>
                        <TableCell className="text-right font-mono">
                          ${formatNumber(r.metrics.total_invested)}
                        </TableCell>
                        <TableCell className="text-right font-mono">
                          {r.metrics.total_trades}
                        </TableCell>
                        <TableCell className="text-right font-mono">
                          ${formatNumber(r.metrics.final_equity)}
                        </TableCell>
                        <TableCell className="text-right font-mono text-muted-foreground">
                          ${formatNumber(r.metrics.total_commission)}
                        </TableCell>
                        <TableCell className="text-right font-mono text-muted-foreground">
                          ${formatNumber(r.metrics.total_tax)}
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </CardContent>
            </Card>

            {/* K-line charts with buy/sell markers per strategy */}
            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-base">
                  {t("backtest.chartTitle")}
                </CardTitle>
              </CardHeader>
              <CardContent>
                <Tabs defaultValue={compareResults[0]?.strategy}>
                  <TabsList>
                    {compareResults.map(({ strategy }) => (
                      <TabsTrigger key={strategy} value={strategy}>
                        {t(`backtest.strategies.${strategy}`)}
                      </TabsTrigger>
                    ))}
                  </TabsList>
                  {compareResults.map(({ strategy, result: r }) => (
                    <TabsContent key={strategy} value={strategy}>
                      <BacktestChart
                        prices={r.prices}
                        trades={r.trades}
                        equityCurve={r.equity_curve}
                      />
                    </TabsContent>
                  ))}
                </Tabs>
              </CardContent>
            </Card>

            {/* Overlaid Equity Curves */}
            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-base">
                  {t("backtest.equityCurve")}
                </CardTitle>
              </CardHeader>
              <CardContent>
                <CompareEquityChart
                  entries={compareResults}
                  initialCapital={initialCapital}
                  t={t}
                />
              </CardContent>
            </Card>

            {/* Per-strategy trade details */}
            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-base">
                  {t("backtest.tradeHistory")}
                </CardTitle>
              </CardHeader>
              <CardContent>
                <Tabs defaultValue={compareResults[0]?.strategy}>
                  <TabsList>
                    {compareResults.map(({ strategy }) => (
                      <TabsTrigger key={strategy} value={strategy}>
                        {t(`backtest.strategies.${strategy}`)}
                      </TabsTrigger>
                    ))}
                  </TabsList>
                  {compareResults.map(({ strategy, result: r }) => (
                    <TabsContent key={strategy} value={strategy}>
                      <TradeTable trades={r.trades} t={t} />
                    </TabsContent>
                  ))}
                </Tabs>
              </CardContent>
            </Card>
          </>
        )}

        {/* Single Result */}
        {result && compareResults.length === 0 && (
          <>
            <BenchmarkBanner benchmark={result.benchmark} symbol={symbol} initialCapital={initialCapital} t={t} />

            <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
              <MetricCard
                label={t("backtest.totalReturn")}
                value={`${result.metrics.total_return_pct >= 0 ? "+" : ""}${result.metrics.total_return_pct.toFixed(2)}%`}
                variant={result.metrics.total_return_pct >= 0 ? "up" : "down"}
              />
              <MetricCard
                label={t("backtest.maxDrawdown")}
                value={`-${result.metrics.max_drawdown_pct.toFixed(2)}%`}
                variant="down"
              />
              <MetricCard
                label={t("backtest.winRate")}
                value={`${result.metrics.win_rate_pct.toFixed(1)}%`}
                variant="neutral"
              />
              <MetricCard
                label={t("backtest.finalEquity")}
                value={`$${formatNumber(result.metrics.final_equity)}`}
                variant="neutral"
              />
            </div>

            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-base">{t("backtest.details")}</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="grid grid-cols-2 md:grid-cols-4 gap-y-2 gap-x-8 text-sm">
                  <DetailRow label={t("backtest.initialCapital")} value={`$${formatNumber(result.metrics.initial_capital)}`} />
                  <DetailRow label={t("backtest.totalInvested")} value={`$${formatNumber(result.metrics.total_invested)}`} />
                  <DetailRow label={t("backtest.finalEquity")} value={`$${formatNumber(result.metrics.final_equity)}`} />
                  <DetailRow label={t("backtest.totalTrades")} value={String(result.metrics.total_trades)} />
                  <DetailRow label={t("backtest.tradingDays")} value={String(result.metrics.trading_days)} />
                  <DetailRow label={t("backtest.winningTrades")} value={String(result.metrics.winning_trades)} className="text-green-600 dark:text-green-400" />
                  <DetailRow label={t("backtest.losingTrades")} value={String(result.metrics.losing_trades)} className="text-red-600 dark:text-red-400" />
                  <DetailRow label={t("backtest.totalCommission")} value={`$${formatNumber(result.metrics.total_commission)}`} />
                  <DetailRow label={t("backtest.totalTax")} value={`$${formatNumber(result.metrics.total_tax)}`} />
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-base">{t("backtest.chartTitle")}</CardTitle>
              </CardHeader>
              <CardContent>
                <BacktestChart
                  prices={result.prices}
                  trades={result.trades}
                  equityCurve={result.equity_curve}
                />
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-base">{t("backtest.tradeHistory")}</CardTitle>
              </CardHeader>
              <CardContent>
                <TradeTable trades={result.trades} t={t} />
              </CardContent>
            </Card>
          </>
        )}
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Strategy-specific parameter inputs
// ---------------------------------------------------------------------------

function StrategyParams({
  config,
  onUpdate,
  t,
}: {
  config: StrategyConfig;
  onUpdate: (key: string, value: number) => void;
  t: (key: string) => string;
}) {
  switch (config.type) {
    case "SmaCrossover":
      return (
        <>
          <ParamInput label={t("backtest.shortMa")} value={config.short_period} onChange={(v) => onUpdate("short_period", v)} min={1} />
          <ParamInput label={t("backtest.longMa")} value={config.long_period} onChange={(v) => onUpdate("long_period", v)} min={2} />
        </>
      );
    case "Rsi":
      return (
        <>
          <ParamInput label={t("backtest.params.period")} value={config.period} onChange={(v) => onUpdate("period", v)} min={2} />
          <ParamInput label={t("backtest.params.overbought")} value={config.overbought} onChange={(v) => onUpdate("overbought", v)} min={50} max={100} />
          <ParamInput label={t("backtest.params.oversold")} value={config.oversold} onChange={(v) => onUpdate("oversold", v)} min={0} max={50} />
        </>
      );
    case "BollingerBands":
      return (
        <>
          <ParamInput label={t("backtest.params.period")} value={config.period} onChange={(v) => onUpdate("period", v)} min={2} />
          <ParamInput label={t("backtest.params.stdDev")} value={config.std_dev} onChange={(v) => onUpdate("std_dev", v)} min={0.5} step={0.1} />
        </>
      );
    case "Macd":
      return (
        <>
          <ParamInput label={t("backtest.params.fastPeriod")} value={config.fast_period} onChange={(v) => onUpdate("fast_period", v)} min={2} />
          <ParamInput label={t("backtest.params.slowPeriod")} value={config.slow_period} onChange={(v) => onUpdate("slow_period", v)} min={2} />
          <ParamInput label={t("backtest.params.signalPeriod")} value={config.signal_period} onChange={(v) => onUpdate("signal_period", v)} min={2} />
        </>
      );
    case "Dca":
      return (
        <>
          <ParamInput label={t("backtest.params.amount")} value={config.amount} onChange={(v) => onUpdate("amount", v)} min={1000} step={1000} />
          <ParamInput label={t("backtest.params.intervalDays")} value={config.interval_days} onChange={(v) => onUpdate("interval_days", v)} min={1} />
        </>
      );
  }
}

function ParamInput({
  label,
  value,
  onChange,
  min,
  max,
  step,
}: {
  label: string;
  value: number;
  onChange: (v: number) => void;
  min?: number;
  max?: number;
  step?: number;
}) {
  return (
    <div className="space-y-1">
      <label className="text-xs text-muted-foreground">{label}</label>
      <Input
        type="number"
        value={value}
        onChange={(e) => onChange(Number(e.target.value))}
        min={min}
        max={max}
        step={step}
      />
    </div>
  );
}

// ---------------------------------------------------------------------------
// Shared sub-components
// ---------------------------------------------------------------------------

function DetailRow({ label, value, className }: { label: string; value: string; className?: string }) {
  return (
    <div className="flex justify-between">
      <span className="text-muted-foreground">{label}</span>
      <span className={className}>{value}</span>
    </div>
  );
}

function BenchmarkBanner({
  benchmark,
  symbol,
  initialCapital,
  t,
}: {
  benchmark: { start_price: number; end_price: number; return_pct: number };
  symbol: string;
  initialCapital: number;
  t: (key: string, opts?: Record<string, unknown>) => string;
}) {
  const up = benchmark.return_pct >= 0;
  const finalValue = initialCapital * (1 + benchmark.return_pct / 100);
  return (
    <Card className={up ? "border-green-500/30 bg-green-500/5" : "border-red-500/30 bg-red-500/5"}>
      <CardContent className="py-3 flex items-center justify-between">
        <div className="text-sm">
          <span className="text-muted-foreground">{t("backtest.benchmark")} </span>
          <span className="font-medium">{symbol}</span>
          <span className="text-muted-foreground">
            {" "}{benchmark.start_price.toFixed(2)} → {benchmark.end_price.toFixed(2)}
          </span>
          <span className="text-muted-foreground ml-3">
            ${formatNumber(initialCapital)} → ${formatNumber(finalValue)}
          </span>
        </div>
        <div className={`text-xl font-bold ${up ? "text-green-600 dark:text-green-400" : "text-red-600 dark:text-red-400"}`}>
          {up ? "+" : ""}{benchmark.return_pct.toFixed(2)}%
        </div>
      </CardContent>
    </Card>
  );
}

function MetricCard({
  label,
  value,
  variant,
}: {
  label: string;
  value: string;
  variant: "up" | "down" | "neutral";
}) {
  const colorClass =
    variant === "up"
      ? "text-green-600 dark:text-green-400"
      : variant === "down"
        ? "text-red-600 dark:text-red-400"
        : "";

  return (
    <Card>
      <CardContent className="pt-4 pb-4">
        <p className="text-xs text-muted-foreground">{label}</p>
        <p className={`text-2xl font-bold ${colorClass}`}>{value}</p>
      </CardContent>
    </Card>
  );
}

function TradeTable({
  trades,
  t,
}: {
  trades: BacktestResult["trades"];
  t: (key: string) => string;
}) {
  return (
    <div className="max-h-80 overflow-auto">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>{t("backtest.date")}</TableHead>
            <TableHead>{t("backtest.action")}</TableHead>
            <TableHead className="text-right">{t("backtest.price")}</TableHead>
            <TableHead className="text-right">{t("backtest.shares")}</TableHead>
            <TableHead className="text-right">{t("backtest.cost")}</TableHead>
            <TableHead className="text-right">{t("backtest.pnl")}</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {trades.length === 0 && (
            <TableRow>
              <TableCell colSpan={6} className="text-center text-muted-foreground">
                {t("backtest.noTrades")}
              </TableCell>
            </TableRow>
          )}
          {trades.map((trade, i) => (
            <TableRow key={i}>
              <TableCell className="font-mono text-xs">{trade.date}</TableCell>
              <TableCell>
                <Badge variant={trade.action === "buy" ? "default" : "secondary"}>
                  {trade.action === "buy" ? t("backtest.buy") : t("backtest.sell")}
                </Badge>
              </TableCell>
              <TableCell className="text-right font-mono">{trade.price.toFixed(2)}</TableCell>
              <TableCell className="text-right font-mono">{formatNumber(trade.shares)}</TableCell>
              <TableCell className="text-right font-mono text-muted-foreground">{formatNumber(trade.cost)}</TableCell>
              <TableCell
                className={`text-right font-mono ${
                  trade.action === "sell"
                    ? trade.pnl >= 0
                      ? "text-green-600 dark:text-green-400"
                      : "text-red-600 dark:text-red-400"
                    : "text-muted-foreground"
                }`}
              >
                {trade.action === "sell"
                  ? `${trade.pnl >= 0 ? "+" : ""}${formatNumber(trade.pnl)}`
                  : "--"}
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Equity Charts
// ---------------------------------------------------------------------------

function sampleCurve(data: { date: string; equity: number }[], maxPoints = 200) {
  const step = Math.max(1, Math.floor(data.length / maxPoints));
  return data.filter((_, i) => i % step === 0 || i === data.length - 1);
}

function CompareEquityChart({
  entries,
  initialCapital,
  t,
}: {
  entries: CompareEntry[];
  initialCapital: number;
  t: (key: string) => string;
}) {
  if (entries.length === 0) return null;

  // Find global min/max across all curves
  let globalMin = initialCapital;
  let globalMax = initialCapital;
  const sampledEntries = entries.map(({ strategy, result: r }) => {
    const sampled = sampleCurve(r.equity_curve);
    for (const d of sampled) {
      if (d.equity < globalMin) globalMin = d.equity;
      if (d.equity > globalMax) globalMax = d.equity;
    }
    return { strategy, sampled };
  });

  const range = globalMax - globalMin || 1;
  const w = 800;
  const h = 240;
  const padY = 10;

  const baseY = padY + (1 - (initialCapital - globalMin) / range) * (h - 2 * padY);

  return (
    <div>
      <svg viewBox={`0 0 ${w} ${h}`} className="w-full h-60">
        {/* Baseline */}
        <line x1={0} y1={baseY} x2={w} y2={baseY} stroke="currentColor" strokeOpacity={0.2} strokeDasharray="4 4" />

        {/* Equity curves */}
        {sampledEntries.map(({ strategy, sampled }) => {
          const points = sampled
            .map((d, i) => {
              const x = (i / (sampled.length - 1)) * w;
              const y = padY + (1 - (d.equity - globalMin) / range) * (h - 2 * padY);
              return `${x},${y}`;
            })
            .join(" ");

          return (
            <polyline
              key={strategy}
              points={points}
              fill="none"
              stroke={STRATEGY_COLORS[strategy]}
              strokeWidth={2}
              strokeOpacity={0.85}
            />
          );
        })}
      </svg>

      {/* Legend */}
      <div className="flex gap-4 mt-2 justify-center">
        {entries.map(({ strategy }) => (
          <div key={strategy} className="flex items-center gap-1.5 text-xs">
            <div
              className="h-2.5 w-2.5 rounded-full"
              style={{ backgroundColor: STRATEGY_COLORS[strategy] }}
            />
            <span>{t(`backtest.strategies.${strategy}`)}</span>
          </div>
        ))}
      </div>
    </div>
  );
}
