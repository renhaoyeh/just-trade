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
import { runBacktest } from "@/services/stockService";
import type { BacktestResult } from "@/types/stock";

function formatNumber(n: number, decimals = 0): string {
  return n.toLocaleString("zh-TW", {
    minimumFractionDigits: decimals,
    maximumFractionDigits: decimals,
  });
}

export default function Backtest() {
  const { t } = useTranslation();

  // Form state
  const [symbol, setSymbol] = useState("2330.TW");
  const [startDate, setStartDate] = useState("2023-01-01");
  const [endDate, setEndDate] = useState("2024-12-31");
  const [initialCapital, setInitialCapital] = useState(1000000);
  const [shortPeriod, setShortPeriod] = useState(5);
  const [longPeriod, setLongPeriod] = useState(20);

  // Result state
  const [result, setResult] = useState<BacktestResult | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleRun = async () => {
    setLoading(true);
    setError(null);
    setResult(null);
    try {
      const res = await runBacktest({
        symbol,
        start_date: startDate,
        end_date: endDate,
        initial_capital: initialCapital,
        short_period: shortPeriod,
        long_period: longPeriod,
      });
      setResult(res);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
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
          <CardContent>
            <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-4">
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
              <div className="space-y-1">
                <label className="text-xs text-muted-foreground">
                  {t("backtest.shortMa")}
                </label>
                <Input
                  type="number"
                  value={shortPeriod}
                  onChange={(e) => setShortPeriod(Number(e.target.value))}
                  min={1}
                />
              </div>
              <div className="space-y-1">
                <label className="text-xs text-muted-foreground">
                  {t("backtest.longMa")}
                </label>
                <Input
                  type="number"
                  value={longPeriod}
                  onChange={(e) => setLongPeriod(Number(e.target.value))}
                  min={2}
                />
              </div>
            </div>
            <div className="mt-4">
              <Button onClick={handleRun} disabled={loading || !symbol}>
                {loading ? t("backtest.running") : t("backtest.run")}
              </Button>
            </div>
            {error && (
              <p className="mt-2 text-sm text-destructive">{error}</p>
            )}
          </CardContent>
        </Card>

        {/* Results */}
        {result && (
          <>
            {/* Metrics Cards */}
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

            {/* Detail Metrics */}
            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-base">
                  {t("backtest.details")}
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="grid grid-cols-2 md:grid-cols-4 gap-y-2 gap-x-8 text-sm">
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">{t("backtest.initialCapital")}</span>
                    <span>${formatNumber(result.metrics.initial_capital)}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">{t("backtest.finalEquity")}</span>
                    <span>${formatNumber(result.metrics.final_equity)}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">{t("backtest.totalTrades")}</span>
                    <span>{result.metrics.total_trades}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">{t("backtest.tradingDays")}</span>
                    <span>{result.metrics.trading_days}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">{t("backtest.winningTrades")}</span>
                    <span className="text-green-600 dark:text-green-400">{result.metrics.winning_trades}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">{t("backtest.losingTrades")}</span>
                    <span className="text-red-600 dark:text-red-400">{result.metrics.losing_trades}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">{t("backtest.totalCommission")}</span>
                    <span>${formatNumber(result.metrics.total_commission)}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">{t("backtest.totalTax")}</span>
                    <span>${formatNumber(result.metrics.total_tax)}</span>
                  </div>
                </div>
              </CardContent>
            </Card>

            {/* Equity Curve */}
            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-base">
                  {t("backtest.equityCurve")}
                </CardTitle>
              </CardHeader>
              <CardContent>
                <EquityChart
                  data={result.equity_curve}
                  initialCapital={result.metrics.initial_capital}
                />
              </CardContent>
            </Card>

            {/* Trade History */}
            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-base">
                  {t("backtest.tradeHistory")}
                </CardTitle>
              </CardHeader>
              <CardContent>
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
                      {result.trades.map((trade, i) => (
                        <TableRow key={i}>
                          <TableCell className="font-mono text-xs">
                            {trade.date}
                          </TableCell>
                          <TableCell>
                            <Badge
                              variant={trade.action === "buy" ? "default" : "secondary"}
                            >
                              {trade.action === "buy" ? t("backtest.buy") : t("backtest.sell")}
                            </Badge>
                          </TableCell>
                          <TableCell className="text-right font-mono">
                            {trade.price.toFixed(2)}
                          </TableCell>
                          <TableCell className="text-right font-mono">
                            {formatNumber(trade.shares)}
                          </TableCell>
                          <TableCell className="text-right font-mono text-muted-foreground">
                            {formatNumber(trade.cost)}
                          </TableCell>
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
              </CardContent>
            </Card>
          </>
        )}
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Sub-components
// ---------------------------------------------------------------------------

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

function EquityChart({
  data,
  initialCapital,
}: {
  data: { date: string; equity: number }[];
  initialCapital: number;
}) {
  if (data.length === 0) return null;

  // Sample data to max ~200 points for SVG performance
  const step = Math.max(1, Math.floor(data.length / 200));
  const sampled = data.filter((_, i) => i % step === 0 || i === data.length - 1);

  const equities = sampled.map((d) => d.equity);
  const minE = Math.min(...equities);
  const maxE = Math.max(...equities);
  const range = maxE - minE || 1;

  const w = 800;
  const h = 200;
  const padX = 0;
  const padY = 10;

  const points = sampled
    .map((d, i) => {
      const x = padX + (i / (sampled.length - 1)) * (w - 2 * padX);
      const y = padY + (1 - (d.equity - minE) / range) * (h - 2 * padY);
      return `${x},${y}`;
    })
    .join(" ");

  // Baseline (initial capital)
  const baseY =
    padY + (1 - (initialCapital - minE) / range) * (h - 2 * padY);

  return (
    <svg viewBox={`0 0 ${w} ${h}`} className="w-full h-48">
      {/* Baseline */}
      <line
        x1={0}
        y1={baseY}
        x2={w}
        y2={baseY}
        stroke="currentColor"
        strokeOpacity={0.2}
        strokeDasharray="4 4"
      />
      {/* Equity curve */}
      <polyline
        points={points}
        fill="none"
        stroke="var(--color-primary)"
        strokeWidth={2}
      />
    </svg>
  );
}
