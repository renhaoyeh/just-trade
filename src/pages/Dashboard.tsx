import { useState, useEffect, useCallback } from "react";
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
import {
  SidebarInset,
  SidebarProvider,
  SidebarTrigger,
} from "@/components/ui/sidebar";
import { TooltipProvider } from "@/components/ui/tooltip";
import { AppSidebar } from "@/components/layout/AppSidebar";
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

export default function Dashboard() {
  const [searchSymbol, setSearchSymbol] = useState("");
  const [selectedSymbol, setSelectedSymbol] = useState("2330.TW");
  const [priceHistory, setPriceHistory] = useState<StockPrice[]>([]);
  const [stockInfo, setStockInfo] = useState<StockInfo | null>(null);
  const [news, setNews] = useState<StockNews[]>([]);
  const [watchlistData, setWatchlistData] = useState<
    { symbol: string; info: StockInfo | null; loading: boolean }[]
  >([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadStockData = useCallback(async (symbol: string) => {
    setLoading(true);
    setError(null);

    const endDate = formatDate(0);
    const startDate = formatDate(90); // 3 months

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
    loadStockData(selectedSymbol);
    loadWatchlist();
  }, [loadStockData, loadWatchlist, selectedSymbol]);

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
    <TooltipProvider>
      <SidebarProvider>
        <AppSidebar />
        <SidebarInset>
          {/* Header */}
          <header className="flex items-center gap-2 border-b px-4 py-3">
            <SidebarTrigger />
            <Separator orientation="vertical" className="h-4" />
            <h1 className="text-sm font-semibold">Dashboard</h1>
            <div className="ml-auto flex items-center gap-2">
              <div className="flex items-center gap-1">
                <Input
                  placeholder="輸入股票代號 (如 2330)"
                  value={searchSymbol}
                  onChange={(e) => setSearchSymbol(e.target.value)}
                  onKeyDown={(e) => e.key === "Enter" && handleSearch()}
                  className="h-8 w-48"
                />
                <Button size="sm" variant="outline" onClick={handleSearch}>
                  查詢
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
                    : loading ? "Loading..." : "N/A"
                }
                description={
                  latestPrice
                    ? `${priceChange >= 0 ? "+" : ""}${priceChange.toFixed(2)} (${priceChangePercent >= 0 ? "+" : ""}${priceChangePercent.toFixed(2)}%)`
                    : ""
                }
                trend={priceChange >= 0 ? "up" : "down"}
              />
              <StatCard
                title="52 Week High"
                value={
                  stockInfo?.fifty_two_week_high
                    ? `$${stockInfo.fifty_two_week_high.toFixed(2)}`
                    : "N/A"
                }
                description={stockInfo?.exchange || ""}
              />
              <StatCard
                title="52 Week Low"
                value={
                  stockInfo?.fifty_two_week_low
                    ? `$${stockInfo.fifty_two_week_low.toFixed(2)}`
                    : "N/A"
                }
                description={
                  stockInfo?.pe_ratio
                    ? `P/E: ${stockInfo.pe_ratio.toFixed(2)}`
                    : ""
                }
              />
              <StatCard
                title="Market Cap"
                value={
                  stockInfo?.market_cap
                    ? stockInfo.market_cap >= 1_000_000_000_000
                      ? `$${(stockInfo.market_cap / 1_000_000_000_000).toFixed(2)}T`
                      : stockInfo.market_cap >= 1_000_000_000
                        ? `$${(stockInfo.market_cap / 1_000_000_000).toFixed(2)}B`
                        : `$${(stockInfo.market_cap / 1_000_000).toFixed(2)}M`
                    : "N/A"
                }
                description={
                  stockInfo?.dividend_yield
                    ? `殖利率: ${(stockInfo.dividend_yield * 100).toFixed(2)}%`
                    : ""
                }
              />
            </div>

            {/* Price Chart */}
            {chartData.length > 0 && (
              <Card>
                <CardHeader>
                  <CardTitle>
                    {selectedSymbol} 股價走勢
                  </CardTitle>
                  <CardDescription>近 3 個月日 K 線收盤價</CardDescription>
                </CardHeader>
                <CardContent>
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
                      <Tooltip />
                      <Line
                        type="monotone"
                        dataKey="close"
                        stroke="hsl(var(--primary))"
                        strokeWidth={2}
                        dot={false}
                      />
                    </LineChart>
                  </ResponsiveContainer>
                </CardContent>
              </Card>
            )}

            {/* Tabs section */}
            <Tabs defaultValue="watchlist">
              <TabsList>
                <TabsTrigger value="watchlist">自選清單</TabsTrigger>
                <TabsTrigger value="history">歷史股價</TabsTrigger>
                <TabsTrigger value="news">相關新聞</TabsTrigger>
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
                            ? "Loading..."
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
                    <CardTitle>{selectedSymbol} 歷史股價</CardTitle>
                    <CardDescription>
                      {priceHistory.length > 0
                        ? `${priceHistory[0].date} ~ ${priceHistory[priceHistory.length - 1].date} (${priceHistory.length} 筆)`
                        : "No data"}
                    </CardDescription>
                  </CardHeader>
                  <CardContent>
                    <Table>
                      <TableHeader>
                        <TableRow>
                          <TableHead>日期</TableHead>
                          <TableHead className="text-right">開盤</TableHead>
                          <TableHead className="text-right">最高</TableHead>
                          <TableHead className="text-right">最低</TableHead>
                          <TableHead className="text-right">收盤</TableHead>
                          <TableHead className="text-right">成交量</TableHead>
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
                    <CardTitle>{selectedSymbol} 相關新聞</CardTitle>
                  </CardHeader>
                  <CardContent className="space-y-4">
                    {news.length === 0 ? (
                      <p className="text-muted-foreground">
                        {loading ? "Loading..." : "暫無新聞"}
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
        </SidebarInset>
      </SidebarProvider>
    </TooltipProvider>
  );
}
