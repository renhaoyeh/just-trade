import { useState, useEffect, useCallback, useRef } from "react";
import { useTranslation } from "react-i18next";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
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
import {
  fugleWsConnect,
  fugleWsDisconnect,
  fugleWsSubscribe,
  fugleWsUnsubscribe,
  getSettings,
} from "@/services/stockService";
import type {
  FugleTrade,
  FugleBook,
  FugleCandle,
  FugleWsStatus,
} from "@/types/stock";

const MAX_TRADES = 50;

function formatTime(iso: string | null): string {
  if (!iso) return "--";
  try {
    const d = new Date(iso);
    return d.toLocaleTimeString("zh-TW", { hour12: false });
  } catch {
    return iso;
  }
}

export default function RealtimeQuotes() {
  const { t } = useTranslation();

  // Connection state
  const [connected, setConnected] = useState(false);
  const [connecting, setConnecting] = useState(false);
  const [statusMsg, setStatusMsg] = useState("");
  const [apiKey, setApiKey] = useState<string | null>(null);

  // Subscription
  const [symbolInput, setSymbolInput] = useState("2330");
  const [subscribedSymbols, setSubscribedSymbols] = useState<string[]>([]);

  // Market data per symbol
  const [trades, setTrades] = useState<Record<string, FugleTrade[]>>({});
  const [books, setBooks] = useState<Record<string, FugleBook>>({});
  const [candles, setCandles] = useState<Record<string, FugleCandle>>({});

  const unlistenRefs = useRef<UnlistenFn[]>([]);

  // Load API key from settings
  useEffect(() => {
    getSettings()
      .then((s) => {
        const fugle = s as unknown as { fugle?: { api_key?: string } };
        if (fugle.fugle?.api_key) setApiKey(fugle.fugle.api_key);
      })
      .catch(console.error);
  }, []);

  // Set up event listeners
  useEffect(() => {
    const setup = async () => {
      const u1 = await listen<FugleWsStatus>("fugle-ws-status", (e) => {
        setConnected(e.payload.connected);
        setStatusMsg(e.payload.message);
        if (!e.payload.connected) setConnecting(false);
      });
      const u2 = await listen<FugleTrade>("fugle-trade", (e) => {
        const trade = e.payload;
        setTrades((prev) => {
          const list = prev[trade.symbol] || [];
          return { ...prev, [trade.symbol]: [trade, ...list].slice(0, MAX_TRADES) };
        });
      });
      const u3 = await listen<FugleBook>("fugle-book", (e) => {
        const book = e.payload;
        setBooks((prev) => ({ ...prev, [book.symbol]: book }));
      });
      const u4 = await listen<FugleCandle>("fugle-candle", (e) => {
        const candle = e.payload;
        setCandles((prev) => ({ ...prev, [candle.symbol]: candle }));
      });
      unlistenRefs.current = [u1, u2, u3, u4];
    };
    setup();
    return () => {
      unlistenRefs.current.forEach((fn) => fn());
    };
  }, []);

  const handleConnect = useCallback(async () => {
    if (!apiKey) return;
    setConnecting(true);
    try {
      await fugleWsConnect(apiKey);
    } catch (e) {
      setStatusMsg(String(e));
      setConnecting(false);
    }
  }, [apiKey]);

  const handleDisconnect = useCallback(async () => {
    try {
      await fugleWsDisconnect();
      setSubscribedSymbols([]);
      setTrades({});
      setBooks({});
      setCandles({});
    } catch (e) {
      setStatusMsg(String(e));
    }
  }, []);

  const handleSubscribe = useCallback(async () => {
    const sym = symbolInput.trim();
    if (!sym || subscribedSymbols.includes(sym)) return;

    try {
      await Promise.all([
        fugleWsSubscribe("trades", sym),
        fugleWsSubscribe("books", sym),
        fugleWsSubscribe("candles", sym),
      ]);
      setSubscribedSymbols((prev) => [...prev, sym]);
      setSymbolInput("");
    } catch (e) {
      setStatusMsg(String(e));
    }
  }, [symbolInput, subscribedSymbols]);

  const handleUnsubscribe = useCallback(
    async (sym: string) => {
      try {
        await Promise.all([
          fugleWsUnsubscribe("trades", sym),
          fugleWsUnsubscribe("books", sym),
          fugleWsUnsubscribe("candles", sym),
        ]);
        setSubscribedSymbols((prev) => prev.filter((s) => s !== sym));
        setTrades((prev) => {
          const next = { ...prev };
          delete next[sym];
          return next;
        });
        setBooks((prev) => {
          const next = { ...prev };
          delete next[sym];
          return next;
        });
        setCandles((prev) => {
          const next = { ...prev };
          delete next[sym];
          return next;
        });
      } catch (e) {
        setStatusMsg(String(e));
      }
    },
    []
  );

  const latestTrade = (sym: string) => trades[sym]?.[0];

  return (
    <>
      {/* Header */}
      <header className="flex items-center gap-2 border-b px-4 py-3">
        <SidebarTrigger />
        <Separator orientation="vertical" className="h-4" />
        <h1 className="text-sm font-semibold">{t("realtime.title")}</h1>
        <div className="ml-auto flex items-center gap-2">
          <Badge variant={connected ? "default" : "outline"}>
            {connected ? t("realtime.connected") : t("realtime.disconnected")}
          </Badge>
          <ThemeToggle />
        </div>
      </header>

      <div className="flex-1 space-y-4 p-6">
        {/* Connection controls */}
        {!apiKey ? (
          <Card className="border-yellow-500">
            <CardContent className="pt-4 text-yellow-600">
              {t("realtime.noApiKey")}
            </CardContent>
          </Card>
        ) : (
          <Card>
            <CardContent className="flex items-center gap-3 pt-4">
              {!connected ? (
                <Button onClick={handleConnect} disabled={connecting}>
                  {connecting ? t("realtime.connecting") : t("realtime.connect")}
                </Button>
              ) : (
                <>
                  <Input
                    placeholder={t("realtime.symbolInput")}
                    value={symbolInput}
                    onChange={(e) => setSymbolInput(e.target.value)}
                    onKeyDown={(e) => e.key === "Enter" && handleSubscribe()}
                    className="h-8 w-40"
                  />
                  <Button size="sm" onClick={handleSubscribe}>
                    {t("realtime.subscribe")}
                  </Button>
                  <div className="ml-auto">
                    <Button size="sm" variant="destructive" onClick={handleDisconnect}>
                      {t("realtime.disconnect")}
                    </Button>
                  </div>
                </>
              )}
            </CardContent>
          </Card>
        )}

        {statusMsg && (
          <p className="text-xs text-muted-foreground">{statusMsg}</p>
        )}

        {/* Subscribed symbol badges */}
        {subscribedSymbols.length > 0 && (
          <div className="flex items-center gap-2 flex-wrap">
            <span className="text-xs text-muted-foreground">{t("realtime.subscribedSymbols")}:</span>
            {subscribedSymbols.map((sym) => (
              <Badge
                key={sym}
                variant="secondary"
                className="cursor-pointer"
                onClick={() => handleUnsubscribe(sym)}
              >
                {sym} &times;
              </Badge>
            ))}
          </div>
        )}

        {/* Data panels per subscribed symbol */}
        {subscribedSymbols.map((sym) => {
          const lt = latestTrade(sym);
          const book = books[sym];
          const candle = candles[sym];
          const tradeList = trades[sym] || [];

          return (
            <div key={sym} className="space-y-4">
              {/* Symbol header with latest price */}
              <div className="flex items-center gap-3">
                <h2 className="text-lg font-bold">{sym}</h2>
                {lt && (
                  <>
                    <span className="text-2xl font-bold tabular-nums">
                      {lt.price?.toFixed(2) ?? "--"}
                    </span>
                    {lt.isLimitUpPrice && (
                      <Badge className="bg-red-600 text-white">{t("realtime.limitUp")}</Badge>
                    )}
                    {lt.isLimitDownPrice && (
                      <Badge className="bg-green-600 text-white">{t("realtime.limitDown")}</Badge>
                    )}
                    {lt.isTrial && (
                      <Badge variant="outline">{t("realtime.trial")}</Badge>
                    )}
                    {lt.isOpen && (
                      <Badge variant="outline" className="text-emerald-500">
                        {t("realtime.marketOpen")}
                      </Badge>
                    )}
                    {lt.isClose && (
                      <Badge variant="outline" className="text-red-400">
                        {t("realtime.marketClosed")}
                      </Badge>
                    )}
                  </>
                )}
              </div>

              <div className="grid gap-4 lg:grid-cols-3">
                {/* Order Book (5 levels) */}
                <Card>
                  <CardHeader className="pb-2">
                    <CardTitle className="text-sm">{t("realtime.orderbook")}</CardTitle>
                  </CardHeader>
                  <CardContent>
                    {book ? (
                      <Table>
                        <TableHeader>
                          <TableRow>
                            <TableHead className="text-right">{t("realtime.bidSize")}</TableHead>
                            <TableHead className="text-right">{t("realtime.bidPrice")}</TableHead>
                            <TableHead className="text-right">{t("realtime.askPrice")}</TableHead>
                            <TableHead className="text-right">{t("realtime.askSize")}</TableHead>
                          </TableRow>
                        </TableHeader>
                        <TableBody>
                          {Array.from({ length: Math.max(book.bids.length, book.asks.length, 5) }).map((_, i) => (
                            <TableRow key={i}>
                              <TableCell className="text-right tabular-nums text-red-400">
                                {book.bids[i]?.size ?? ""}
                              </TableCell>
                              <TableCell className="text-right tabular-nums font-medium text-red-400">
                                {book.bids[i]?.price?.toFixed(2) ?? ""}
                              </TableCell>
                              <TableCell className="text-right tabular-nums font-medium text-green-400">
                                {book.asks[i]?.price?.toFixed(2) ?? ""}
                              </TableCell>
                              <TableCell className="text-right tabular-nums text-green-400">
                                {book.asks[i]?.size ?? ""}
                              </TableCell>
                            </TableRow>
                          ))}
                        </TableBody>
                      </Table>
                    ) : (
                      <p className="text-xs text-muted-foreground">{t("realtime.noData")}</p>
                    )}
                  </CardContent>
                </Card>

                {/* Candle (intraday OHLC) */}
                <Card>
                  <CardHeader className="pb-2">
                    <CardTitle className="text-sm">{t("realtime.candles")}</CardTitle>
                  </CardHeader>
                  <CardContent>
                    {candle ? (
                      <div className="grid grid-cols-2 gap-y-2 gap-x-4 text-sm">
                        <span className="text-muted-foreground">{t("realtime.open")}</span>
                        <span className="text-right tabular-nums font-medium">{candle.open?.toFixed(2) ?? "--"}</span>
                        <span className="text-muted-foreground">{t("realtime.high")}</span>
                        <span className="text-right tabular-nums font-medium text-red-400">{candle.high?.toFixed(2) ?? "--"}</span>
                        <span className="text-muted-foreground">{t("realtime.low")}</span>
                        <span className="text-right tabular-nums font-medium text-green-400">{candle.low?.toFixed(2) ?? "--"}</span>
                        <span className="text-muted-foreground">{t("realtime.close")}</span>
                        <span className="text-right tabular-nums font-medium">{candle.close?.toFixed(2) ?? "--"}</span>
                        <span className="text-muted-foreground">{t("realtime.volume")}</span>
                        <span className="text-right tabular-nums">{candle.volume?.toLocaleString() ?? "--"}</span>
                        <span className="text-muted-foreground">{t("realtime.average")}</span>
                        <span className="text-right tabular-nums">{candle.average?.toFixed(2) ?? "--"}</span>
                        <span className="text-muted-foreground">{t("realtime.time")}</span>
                        <span className="text-right tabular-nums text-xs">{formatTime(candle.time)}</span>
                      </div>
                    ) : (
                      <p className="text-xs text-muted-foreground">{t("realtime.noData")}</p>
                    )}
                  </CardContent>
                </Card>

                {/* Recent Trades */}
                <Card>
                  <CardHeader className="pb-2">
                    <CardTitle className="text-sm">{t("realtime.trades")}</CardTitle>
                  </CardHeader>
                  <CardContent className="max-h-64 overflow-y-auto">
                    {tradeList.length > 0 ? (
                      <Table>
                        <TableHeader>
                          <TableRow>
                            <TableHead className="text-right">{t("realtime.time")}</TableHead>
                            <TableHead className="text-right">{t("realtime.price")}</TableHead>
                            <TableHead className="text-right">{t("realtime.size")}</TableHead>
                            <TableHead className="text-right">{t("realtime.totalVolume")}</TableHead>
                          </TableRow>
                        </TableHeader>
                        <TableBody>
                          {tradeList.map((tr, i) => (
                            <TableRow key={`${tr.serial}-${i}`}>
                              <TableCell className="text-right tabular-nums text-xs">
                                {formatTime(tr.time)}
                              </TableCell>
                              <TableCell className="text-right tabular-nums font-medium">
                                {tr.price?.toFixed(2) ?? "--"}
                              </TableCell>
                              <TableCell className="text-right tabular-nums">
                                {tr.size ?? "--"}
                              </TableCell>
                              <TableCell className="text-right tabular-nums text-muted-foreground">
                                {tr.volume?.toLocaleString() ?? "--"}
                              </TableCell>
                            </TableRow>
                          ))}
                        </TableBody>
                      </Table>
                    ) : (
                      <p className="text-xs text-muted-foreground">{t("realtime.noData")}</p>
                    )}
                  </CardContent>
                </Card>
              </div>

              <Separator />
            </div>
          );
        })}
      </div>
    </>
  );
}
