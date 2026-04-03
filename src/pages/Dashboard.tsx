import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { useSearchParams } from "react-router";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Separator } from "@/components/ui/separator";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { SidebarTrigger } from "@/components/ui/sidebar";
import { ThemeToggle } from "@/components/common/ThemeToggle";
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from "recharts";

import type { StockPrice, StockInfo, StockNews } from "@/types/stock";
import {
  fetchStockHistory,
  getStockInfo,
  fetchStockNews,
} from "@/services/stockService";

// Default Taiwan stock watchlist
const DEFAULT_WATCHLIST = ["2330.TW", "2317.TW", "2454.TW", "2308.TW"];

function StatCard({
  title,
  value,
  description,
  trend,
}: {
  title: string;
  value: string;
  description: string;
  trend?: "up" | "down";
}) {
  return (
    <Card>
      <CardHeader>
        <CardDescription>{title}</CardDescription>
        <CardTitle className="text-2xl tabular-nums">{value}</CardTitle>
      </CardHeader>
      <CardContent>
        <span
          className={
            trend === "up"
              ? "text-emerald-500"
              : trend === "down"
                ? "text-red-500"
                : "text-muted-foreground"
          }
        >
          {description}
        </span>
      </CardContent>
    </Card>
  );
}

function formatDate(daysAgo: number): string {
  const d = new Date();
  d.setDate(d.getDate() - daysAgo);
  return d.toISOString().split("T")[0];
}

const RANGE_OPTIONS = [
  { label: "1M", days: 30 },
  { label: "3M", days: 90 },
  { label: "6M", days: 180 },
  { label: "1Y", days: 365 },
  { label: "5Y", days: 1825 },
] as const;

type RangeLabel = (typeof RANGE_OPTIONS)[number]["label"];

export default function Dashboard() {
  const [searchParams] = useSearchParams();
  const [searchSymbol, setSearchSymbol] = useState("");
  const [selectedSymbol, setSelectedSymbol] = useState(
    searchParams.get("symbol") || "2330.TW"
  );
  const [selectedRange, setSelectedRange] = useState<RangeLabel>("3M");
  const [priceHistory, setPriceHistory] = useState<StockPrice[]>([]);
  const [stockInfo, setStockInfo] = useState<StockInfo | null>(null);
  const [news, setNews] = useState<StockNews[]>([]);
  const [watchlistData, setWatchlistData] = useState<
    { symbol: string; info: StockInfo | null; loading: boolean }[]
  >([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const { t } = useTranslation();
  const rangeConfig = RANGE_OPTIONS.find((r) => r.label === selectedRange)!;

  const loadStockData = useCallback(async (symbol: string, days: number) => {
    setLoading(true);
    setError(null);

    const endDate = formatDate(0);
    const startDate = formatDate(days);

    try {
      const [history, info, stockNews] = await Promise.all([
        fetchStockHistory(symbol, startDate, endDate),
        getStockInfo(symbol).catch(() => null),
        fetchStockNews(symbol, 5).catch(() => []),
      ]);
      setPriceHistory(history);
      setStockInfo(info);
      setNews(stockNews);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  const loadWatchlist = useCallback(async () => {
    setWatchlistData(
      DEFAULT_WATCHLIST.map((s) => ({ symbol: s, info: null, loading: true }))
    );

    for (const symbol of DEFAULT_WATCHLIST) {
      try {
        const info = await getStockInfo(symbol);
        setWatchlistData((prev) =>
          prev.map((item) =>
            item.symbol === symbol ? { ...item, info, loading: false } : item
          )
        );
      } catch {
        setWatchlistData((prev) =>
          prev.map((item) =>
            item.symbol === symbol ? { ...item, loading: false } : item
          )
        );
      }
    }
  }, []);

  useEffect(() => {
    loadStockData(selectedSymbol, rangeConfig.days);
    loadWatchlist();
  }, [loadStockData, loadWatchlist, selectedSymbol, rangeConfig.days]);

  const handleSearch = () => {
    const symbol = searchSymbol.trim().toUpperCase();
    if (!symbol) return;
    // Auto-append .TW if no suffix
    const fullSymbol =
      symbol.includes(".") ? symbol : `${symbol}.TW`;
    setSelectedSymbol(fullSymbol);
    setSearchSymbol("");
  };

  // Chart data
  const chartData = priceHistory.map((p) => ({
    date: p.date,
    close: p.close,
    volume: p.volume,
  }));

  // Latest price info
  const latestPrice = priceHistory.length > 0 ? priceHistory[priceHistory.length - 1] : null;
  const prevPrice = priceHistory.length > 1 ? priceHistory[priceHistory.length - 2] : null;
  const priceChange = latestPrice && prevPrice ? latestPrice.close - prevPrice.close : 0;
  const priceChangePercent = prevPrice ? (priceChange / prevPrice.close) * 100 : 0;

  return (
    <>
      {/* Header */}
          <header className="flex items-center gap-2 border-b px-4 py-3">
            <SidebarTrigger />
            <Separator orientation="vertical" className="h-4" />
            <h1 className="text-sm font-semibold">{t("dashboard.title")}</h1>
            <div className="ml-auto flex items-center gap-2">
              <div className="flex items-center gap-1">
                <Input
                  placeholder={t("dashboard.searchPlaceholder")}
                  value={searchSymbol}
                  onChange={(e) => setSearchSymbol(e.target.value)}
                  onKeyDown={(e) => e.key === "Enter" && handleSearch()}
                  className="h-8 w-48"
                />
                <Button size="sm" variant="outline" onClick={handleSearch}>
                  {t("dashboard.search")}
                </Button>
              </div>
              <Badge variant="outline">
                {selectedSymbol}
              </Badge>
              <ThemeToggle />
            </div>
          </header>

          {/* Main content */}
          <div className="flex-1 space-y-6 p-6">
            {error && (
              <Card className="border-red-500">
                <CardContent className="pt-4 text-red-500">{error}</CardContent>
              </Card>
            )}

            {/* Stats row */}
            <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
              <StatCard
                title={stockInfo?.long_name || stockInfo?.short_name || selectedSymbol}
                value={
                  latestPrice
                    ? `${stockInfo?.currency === "TWD" ? "NT$" : "$"}${latestPrice.close.toFixed(2)}`
                    : loading ? t("dashboard.loading") : t("dashboard.na")
                }
                description={
                  latestPrice
                    ? `${priceChange >= 0 ? "+" : ""}${priceChange.toFixed(2)} (${priceChangePercent >= 0 ? "+" : ""}${priceChangePercent.toFixed(2)}%)`
                    : ""
                }
                trend={priceChange >= 0 ? "up" : "down"}
              />
              <StatCard
                title={t("dashboard.weekHigh52")}
                value={
                  stockInfo?.fifty_two_week_high
                    ? `$${stockInfo.fifty_two_week_high.toFixed(2)}`
                    : t("dashboard.na")
                }
                description={stockInfo?.exchange || ""}
              />
              <StatCard
                title={t("dashboard.weekLow52")}
                value={
                  stockInfo?.fifty_two_week_low
                    ? `$${stockInfo.fifty_two_week_low.toFixed(2)}`
                    : t("dashboard.na")
                }
                description={
                  stockInfo?.pe_ratio
                    ? t("dashboard.peRatio", { value: stockInfo.pe_ratio.toFixed(2) })
                    : ""
                }
              />
              <StatCard
                title={t("dashboard.marketCap")}
                value={
                  stockInfo?.market_cap
                    ? stockInfo.market_cap >= 1_000_000_000_000
                      ? `$${(stockInfo.market_cap / 1_000_000_000_000).toFixed(2)}T`
                      : stockInfo.market_cap >= 1_000_000_000
                        ? `$${(stockInfo.market_cap / 1_000_000_000).toFixed(2)}B`
                        : `$${(stockInfo.market_cap / 1_000_000).toFixed(2)}M`
                    : t("dashboard.na")
                }
                description={
                  stockInfo?.dividend_yield
                    ? t("dashboard.dividendYield", { value: (stockInfo.dividend_yield * 100).toFixed(2) })
                    : ""
                }
              />
            </div>

            {/* Price Chart */}
            <Card>
              <CardHeader>
                <div className="flex items-center justify-between">
                  <div>
                    <CardTitle>
                      {t("chart.title", { symbol: selectedSymbol })}
                    </CardTitle>
                    <CardDescription>{t("chart.description", { range: t(`chart.range.${selectedRange}`) })}</CardDescription>
                  </div>
                  <div className="flex gap-1">
                    {RANGE_OPTIONS.map((opt) => (
                      <Button
                        key={opt.label}
                        size="sm"
                        variant={selectedRange === opt.label ? "default" : "outline"}
                        onClick={() => setSelectedRange(opt.label)}
                        className="h-7 px-2 text-xs"
                      >
                        {opt.label}
                      </Button>
                    ))}
                  </div>
                </div>
              </CardHeader>
              <CardContent>
                {loading && chartData.length === 0 ? (
                  <div className="flex h-75 items-center justify-center text-muted-foreground">
                    {t("chart.loading")}
                  </div>
                ) : chartData.length === 0 ? (
                  <div className="flex h-75 items-center justify-center text-muted-foreground">
                    {t("chart.empty")}
                  </div>
                ) : (
                  <ResponsiveContainer width="100%" height={300}>
                    <LineChart data={chartData}>
                      <CartesianGrid strokeDasharray="3 3" />
                      <XAxis
                        dataKey="date"
                        tick={{ fontSize: 12 }}
                        interval="preserveStartEnd"
                      />
                      <YAxis
                        domain={["auto", "auto"]}
                        tick={{ fontSize: 12 }}
                      />
                      <Tooltip
                        contentStyle={{
                          backgroundColor: "var(--color-popover)",
                          color: "var(--color-popover-foreground)",
                          border: "1px solid var(--color-border)",
                          borderRadius: 0,
                        }}
                      />
                      <Line
                        type="monotone"
                        dataKey="close"
                        stroke="var(--color-primary)"
                        strokeWidth={2}
                        dot={false}
                      />
                    </LineChart>
                  </ResponsiveContainer>
                )}
              </CardContent>
            </Card>

            {/* Tabs section */}
            <Tabs defaultValue="watchlist">
              <TabsList>
                <TabsTrigger value="watchlist">{t("tabs.watchlist")}</TabsTrigger>
                <TabsTrigger value="history">{t("tabs.history")}</TabsTrigger>
                <TabsTrigger value="news">{t("tabs.news")}</TabsTrigger>
              </TabsList>

              {/* Watchlist tab */}
              <TabsContent value="watchlist">
                <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
                  {watchlistData.map((stock) => (
                    <Card
                      key={stock.symbol}
                      className="cursor-pointer transition-colors hover:bg-muted/50"
                      onClick={() => setSelectedSymbol(stock.symbol)}
                    >
                      <CardHeader>
                        <CardTitle className="flex items-center justify-between">
                          {stock.symbol.replace(".TW", "").replace(".TWO", "")}
                          {stock.info && (
                            <Badge variant="outline" className="text-xs">
                              {stock.info.exchange}
                            </Badge>
                          )}
                        </CardTitle>
                        <CardDescription>
                          {stock.loading
                            ? t("dashboard.loading")
                            : stock.info?.short_name || stock.info?.long_name || ""}
                        </CardDescription>
                      </CardHeader>
                      <CardContent>
                        {stock.info?.fifty_two_week_high && (
                          <div className="text-sm text-muted-foreground">
                            52W: ${stock.info.fifty_two_week_low?.toFixed(2)} - $
                            {stock.info.fifty_two_week_high.toFixed(2)}
                          </div>
                        )}
                        {stock.info?.pe_ratio && (
                          <div className="text-sm text-muted-foreground">
                            P/E: {stock.info.pe_ratio.toFixed(2)}
                          </div>
                        )}
                      </CardContent>
                    </Card>
                  ))}
                </div>
              </TabsContent>

              {/* History tab */}
              <TabsContent value="history">
                <Card>
                  <CardHeader>
                    <CardTitle>{t("history.title", { symbol: selectedSymbol })}</CardTitle>
                    <CardDescription>
                      {priceHistory.length > 0
                        ? `${priceHistory[0].date} ~ ${priceHistory[priceHistory.length - 1].date} (${priceHistory.length} 筆)`
                        : t("dashboard.noData")}
                    </CardDescription>
                  </CardHeader>
                  <CardContent>
                    <Table>
                      <TableHeader>
                        <TableRow>
                          <TableHead>{t("history.date")}</TableHead>
                          <TableHead className="text-right">{t("history.open")}</TableHead>
                          <TableHead className="text-right">{t("history.high")}</TableHead>
                          <TableHead className="text-right">{t("history.low")}</TableHead>
                          <TableHead className="text-right">{t("history.close")}</TableHead>
                          <TableHead className="text-right">{t("history.volume")}</TableHead>
                        </TableRow>
                      </TableHeader>
                      <TableBody>
                        {[...priceHistory].reverse().slice(0, 30).map((price) => (
                          <TableRow key={price.date}>
                            <TableCell className="tabular-nums">
                              {price.date}
                            </TableCell>
                            <TableCell className="text-right tabular-nums">
                              {price.open.toFixed(2)}
                            </TableCell>
                            <TableCell className="text-right tabular-nums">
                              {price.high.toFixed(2)}
                            </TableCell>
                            <TableCell className="text-right tabular-nums">
                              {price.low.toFixed(2)}
                            </TableCell>
                            <TableCell className="text-right tabular-nums">
                              {price.close.toFixed(2)}
                            </TableCell>
                            <TableCell className="text-right tabular-nums">
                              {(price.volume / 1000).toFixed(0)}K
                            </TableCell>
                          </TableRow>
                        ))}
                      </TableBody>
                    </Table>
                  </CardContent>
                </Card>
              </TabsContent>

              {/* News tab */}
              <TabsContent value="news">
                <Card>
                  <CardHeader>
                    <CardTitle>{t("news.title", { symbol: selectedSymbol })}</CardTitle>
                  </CardHeader>
                  <CardContent className="space-y-4">
                    {news.length === 0 ? (
                      <p className="text-muted-foreground">
                        {loading ? t("dashboard.loading") : t("news.empty")}
                      </p>
                    ) : (
                      news.map((item, i) => (
                        <div key={i} className="border-b pb-3 last:border-0">
                          <div className="flex items-start justify-between gap-2">
                            <div>
                              {item.link ? (
                                <a
                                  href={item.link}
                                  target="_blank"
                                  rel="noopener noreferrer"
                                  className="font-medium hover:underline"
                                >
                                  {item.title}
                                </a>
                              ) : (
                                <span className="font-medium">{item.title}</span>
                              )}
                              {item.summary && (
                                <p className="mt-1 text-sm text-muted-foreground">
                                  {item.summary}
                                </p>
                              )}
                            </div>
                            <Badge variant="outline" className="shrink-0 text-xs">
                              {item.publisher}
                            </Badge>
                          </div>
                          {item.pub_date && (
                            <p className="mt-1 text-xs text-muted-foreground">
                              {item.pub_date}
                            </p>
                          )}
                        </div>
                      ))
                    )}
                  </CardContent>
                </Card>
              </TabsContent>
            </Tabs>
          </div>
    </>
  );
}
