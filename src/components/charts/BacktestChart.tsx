import { useEffect, useRef } from "react";
import {
  createChart,
  ColorType,
  CrosshairMode,
  CandlestickSeries,
  HistogramSeries,
  LineSeries,
  createSeriesMarkers,
  type IChartApi,
} from "lightweight-charts";
import type { BacktestTrade, PriceBar, EquityPoint } from "@/types/stock";

interface BacktestChartProps {
  prices: PriceBar[];
  trades: BacktestTrade[];
  equityCurve: EquityPoint[];
  height?: number;
}

export function BacktestChart({
  prices,
  trades,
  equityCurve,
  height = 500,
}: BacktestChartProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const chartRef = useRef<IChartApi | null>(null);

  useEffect(() => {
    if (!containerRef.current || prices.length === 0) return;

    const isDark = document.documentElement.classList.contains("dark");
    const bg = isDark ? "#09090b" : "#ffffff";
    const textColor = isDark ? "#a1a1aa" : "#71717a";
    const gridColor = isDark ? "#27272a" : "#e4e4e7";

    const chart = createChart(containerRef.current, {
      layout: {
        background: { type: ColorType.Solid, color: bg },
        textColor,
      },
      grid: {
        vertLines: { color: gridColor },
        horzLines: { color: gridColor },
      },
      crosshair: { mode: CrosshairMode.Normal },
      rightPriceScale: { borderColor: gridColor },
      timeScale: {
        borderColor: gridColor,
        tickMarkFormatter: (time: string) => {
          const d = new Date(time);
          const mm = String(d.getMonth() + 1).padStart(2, "0");
          const dd = String(d.getDate()).padStart(2, "0");
          return `${mm}/${dd}`;
        },
      },
      localization: { dateFormat: "yyyy/MM/dd" },
      height,
      autoSize: true,
    });

    // Candlestick series (台股: red = up, green = down)
    const candleSeries = chart.addSeries(CandlestickSeries, {
      upColor: "#ef4444",
      downColor: "#22c55e",
      borderUpColor: "#ef4444",
      borderDownColor: "#22c55e",
      wickUpColor: "#ef4444",
      wickDownColor: "#22c55e",
    });

    const candleData = prices.map((p) => ({
      time: p.date as string,
      open: p.open,
      high: p.high,
      low: p.low,
      close: p.close,
    }));
    candleSeries.setData(candleData);

    // Buy/Sell markers on candles
    const markers = trades.map((t) => ({
      time: t.date as string,
      position: t.action === "buy" ? ("belowBar" as const) : ("aboveBar" as const),
      color: t.action === "buy" ? "#3b82f6" : "#f59e0b",
      shape: t.action === "buy" ? ("arrowUp" as const) : ("arrowDown" as const),
      text: t.action === "buy" ? "B" : "S",
    }));
    // Sort markers by time (required by lightweight-charts)
    markers.sort((a, b) => a.time.localeCompare(b.time));
    createSeriesMarkers(candleSeries, markers);

    // Volume histogram
    const volumeSeries = chart.addSeries(HistogramSeries, {
      priceFormat: { type: "volume" },
      priceScaleId: "volume",
    });
    chart.priceScale("volume").applyOptions({
      scaleMargins: { top: 0.85, bottom: 0 },
    });
    volumeSeries.setData(
      prices.map((p) => ({
        time: p.date as string,
        value: p.volume,
        color: p.close >= p.open ? "#ef444440" : "#22c55e40",
      }))
    );

    // Equity curve as overlay line
    const equitySeries = chart.addSeries(LineSeries, {
      color: "#8b5cf6",
      lineWidth: 2,
      priceScaleId: "equity",
      lastValueVisible: true,
      priceLineVisible: false,
    });
    chart.priceScale("equity").applyOptions({
      scaleMargins: { top: 0.05, bottom: 0.2 },
    });
    equitySeries.setData(
      equityCurve.map((e) => ({
        time: e.date as string,
        value: e.equity,
      }))
    );

    chart.timeScale().fitContent();
    chartRef.current = chart;

    // Theme observer
    const observer = new MutationObserver(() => {
      if (!chartRef.current) return;
      const dark = document.documentElement.classList.contains("dark");
      const newBg = dark ? "#09090b" : "#ffffff";
      const newText = dark ? "#a1a1aa" : "#71717a";
      const newGrid = dark ? "#27272a" : "#e4e4e7";
      chartRef.current.applyOptions({
        layout: { background: { type: ColorType.Solid, color: newBg }, textColor: newText },
        grid: { vertLines: { color: newGrid }, horzLines: { color: newGrid } },
      });
    });
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["class"],
    });

    return () => {
      observer.disconnect();
      chart.remove();
      chartRef.current = null;
    };
  }, [prices, trades, equityCurve, height]);

  return <div ref={containerRef} />;
}
