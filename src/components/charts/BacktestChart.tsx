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
  type ISeriesMarkersPluginApi,
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

// ---------------------------------------------------------------------------
// Multi-strategy chart: one K-line with toggleable markers per strategy
// ---------------------------------------------------------------------------

export interface StrategyTradeEntry {
  name: string;
  color: string;
  trades: BacktestTrade[];
  enabled: boolean;
}

interface CompareBacktestChartProps {
  prices: PriceBar[];
  strategies: StrategyTradeEntry[];
  onToggle: (index: number) => void;
  height?: number;
}

export function CompareBacktestChart({
  prices,
  strategies,
  onToggle,
  height = 500,
}: CompareBacktestChartProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const chartRef = useRef<IChartApi | null>(null);
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const markersPluginRef = useRef<ISeriesMarkersPluginApi<any> | null>(null);
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const candleSeriesRef = useRef<any>(null);

  // Create chart once
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

    const candleSeries = chart.addSeries(CandlestickSeries, {
      upColor: "#ef4444",
      downColor: "#22c55e",
      borderUpColor: "#ef4444",
      borderDownColor: "#22c55e",
      wickUpColor: "#ef4444",
      wickDownColor: "#22c55e",
    });
    candleSeries.setData(
      prices.map((p) => ({
        time: p.date as string,
        open: p.open,
        high: p.high,
        low: p.low,
        close: p.close,
      }))
    );

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

    const plugin = createSeriesMarkers(candleSeries, []);
    markersPluginRef.current = plugin;
    candleSeriesRef.current = candleSeries;

    chart.timeScale().fitContent();
    chartRef.current = chart;

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
      markersPluginRef.current = null;
      candleSeriesRef.current = null;
    };
  }, [prices, height]);

  // Update markers when strategies toggle
  useEffect(() => {
    if (!markersPluginRef.current) return;

    const allMarkers: {
      time: string;
      position: "belowBar" | "aboveBar";
      color: string;
      shape: "arrowUp" | "arrowDown";
      text: string;
    }[] = [];

    for (const s of strategies) {
      if (!s.enabled) continue;
      for (const t of s.trades) {
        allMarkers.push({
          time: t.date,
          position: t.action === "buy" ? "belowBar" : "aboveBar",
          color: s.color,
          shape: t.action === "buy" ? "arrowUp" : "arrowDown",
          text: t.action === "buy" ? "B" : "S",
        });
      }
    }

    allMarkers.sort((a, b) => a.time.localeCompare(b.time));
    markersPluginRef.current.setMarkers(allMarkers);
  }, [strategies]);

  return (
    <div>
      <div ref={containerRef} />
      <div className="flex flex-wrap gap-4 mt-3">
        {strategies.map((s, i) => (
          <label key={i} className="flex items-center gap-1.5 text-sm cursor-pointer select-none">
            <input
              type="checkbox"
              checked={s.enabled}
              onChange={() => onToggle(i)}
              className="accent-current"
              style={{ accentColor: s.color }}
            />
            <div
              className="h-2.5 w-2.5 rounded-full"
              style={{ backgroundColor: s.color }}
            />
            <span>{s.name}</span>
          </label>
        ))}
      </div>
    </div>
  );
}
