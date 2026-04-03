import { useState, useEffect, useCallback, useRef } from "react";
import Markdown from "react-markdown";
import { useTranslation } from "react-i18next";
import { listen } from "@tauri-apps/api/event";
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
import { Switch } from "@/components/ui/switch";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Skeleton } from "@/components/ui/skeleton";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import {
  runAnalysis,
  getSettings,
  getAnalysisHistory,
  getAnalysisDetail,
  type PipelineConfig,
  type LlmConfig,
  type AnalysisRecord,
} from "@/services/stockService";

interface ProgressEvent {
  phase: string;
  agent: string;
  status: string;
  message: string | null;
}

interface StepStatus {
  agent: string;
  phase: string;
  status: "pending" | "running" | "done" | "error";
  statusMessage?: string;
  content?: string;
}

const ALL_STEPS: { agent: string; phase: string }[] = [
  { agent: "Market Analyst", phase: "analysts" },
  { agent: "News Analyst", phase: "analysts" },
  { agent: "Fundamentals Analyst", phase: "analysts" },
  { agent: "Social Media Analyst", phase: "analysts" },
  { agent: "Bull Researcher", phase: "debate" },
  { agent: "Bear Researcher", phase: "debate" },
  { agent: "Research Manager", phase: "decision" },
  { agent: "Trader", phase: "decision" },
  { agent: "Aggressive Analyst", phase: "risk" },
  { agent: "Conservative Analyst", phase: "risk" },
  { agent: "Neutral Analyst", phase: "risk" },
  { agent: "Portfolio Manager", phase: "final" },
];

const PHASE_ORDER = ["analysts", "debate", "decision", "risk", "final"];
const PHASE_LABELS: Record<string, string> = {
  analysts: "analysis.phaseAnalysts",
  debate: "analysis.phaseDebate",
  decision: "analysis.phaseDecision",
  risk: "analysis.phaseRisk",
  final: "analysis.phaseFinal",
};

function stepKey(s: { phase: string; agent: string }) {
  return `${s.phase}-${s.agent}`;
}

function groupByPhase(steps: StepStatus[]): Record<string, StepStatus[]> {
  const groups: Record<string, StepStatus[]> = {};
  for (const step of steps) {
    if (!groups[step.phase]) groups[step.phase] = [];
    groups[step.phase].push(step);
  }
  return groups;
}

export default function Analysis() {
  const { t } = useTranslation();
  const [symbol, setSymbol] = useState("2330.TW");
  const [running, setRunning] = useState(false);
  const [steps, setSteps] = useState<StepStatus[]>([]);
  const [signal, setSignal] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [expandedCards, setExpandedCards] = useState<Set<string>>(new Set());
  const [history, setHistory] = useState<AnalysisRecord[]>([]);
  const bottomRef = useRef<HTMLDivElement>(null);

  // Config
  const [debateRounds, setDebateRounds] = useState(1);
  const [riskRounds, setRiskRounds] = useState(1);
  const [cooldownSecs, setCooldownSecs] = useState(15);
  const [enableMarket, setEnableMarket] = useState(true);
  const [enableNews, setEnableNews] = useState(true);
  const [enableFundamentals, setEnableFundamentals] = useState(true);
  const [enableSocial, setEnableSocial] = useState(false);

  // Load history on mount
  useEffect(() => {
    getAnalysisHistory(20).then(setHistory).catch(console.error);
  }, []);

  // Listen to progress events
  useEffect(() => {
    const unlisten = listen<ProgressEvent>("analysis-progress", (event) => {
      const { phase, agent, status, message } = event.payload;

      if (phase === "complete") {
        setSignal(message || "HOLD");
        getAnalysisHistory(20).then(setHistory).catch(console.error);
        return;
      }

      if (phase === "cooldown") return;

      setSteps((prev) => {
        const key = `${phase}-${agent}`;
        const existing = prev.find((s) => stepKey(s) === key);

        const updated: StepStatus = existing
          ? { ...existing }
          : { agent, phase, status: "pending" };

        if (status === "running") {
          updated.status = "running";
          updated.statusMessage = message || undefined;
        } else if (status === "done") {
          updated.status = "done";
          updated.content = message || undefined;
          // Auto-expand
          setExpandedCards((prev) => new Set(prev).add(key));
        } else if (status === "error") {
          updated.status = "error";
          updated.statusMessage = message || undefined;
        }

        if (existing) {
          return prev.map((s) => (stepKey(s) === key ? updated : s));
        }
        return [...prev, updated];
      });

      // Auto-scroll
      setTimeout(() => bottomRef.current?.scrollIntoView({ behavior: "smooth" }), 100);
    });

    return () => { unlisten.then((fn) => fn()); };
  }, []);

  const handleRun = useCallback(async () => {
    setRunning(true);
    setSteps([]);
    setSignal(null);
    setError(null);
    setExpandedCards(new Set());

    let quickLlm: LlmConfig = { provider: "openai", model: "gpt-4o-mini" };
    let deepLlm: LlmConfig = { provider: "openai", model: "gpt-4o" };

    try {
      const settings = await getSettings();
      for (const [provider, cfg] of Object.entries(settings)) {
        if (cfg && cfg.api_key && cfg.model) {
          quickLlm = { provider, model: cfg.model, api_key: cfg.api_key, base_url: cfg.base_url };
          deepLlm = { ...quickLlm };
          break;
        }
        if (provider === "ollama" && cfg && cfg.base_url && cfg.model) {
          quickLlm = { provider: "ollama", model: cfg.model, base_url: cfg.base_url };
          deepLlm = { ...quickLlm };
          break;
        }
      }
    } catch { /* use defaults */ }

    const config: PipelineConfig = {
      quick_llm: quickLlm,
      deep_llm: deepLlm,
      max_debate_rounds: debateRounds,
      max_risk_rounds: riskRounds,
      enable_market_analyst: enableMarket,
      enable_news_analyst: enableNews,
      enable_fundamentals_analyst: enableFundamentals,
      enable_social_analyst: enableSocial,
      cooldown_secs: cooldownSecs,
    };

    try {
      await runAnalysis(symbol, config);
    } catch (e) {
      setError(String(e));
    } finally {
      setRunning(false);
    }
  }, [symbol, debateRounds, riskRounds, enableMarket, enableNews, enableFundamentals, enableSocial, cooldownSecs]);

  const loadHistory = async (record: AnalysisRecord) => {
    try {
      const detail = await getAnalysisDetail(record.id) as Record<string, string>;
      setSignal(record.signal);
      setSteps([]);
      setExpandedCards(new Set());

      // Map AnalysisResult fields back to steps
      const fieldMap: [string, string, string][] = [
        ["market_report", "Market Analyst", "analysts"],
        ["news_report", "News Analyst", "analysts"],
        ["fundamentals_report", "Fundamentals Analyst", "analysts"],
        ["social_report", "Social Media Analyst", "analysts"],
        ["bull_arguments", "Bull Researcher", "debate"],
        ["bear_arguments", "Bear Researcher", "debate"],
        ["investment_decision", "Research Manager", "decision"],
        ["trader_plan", "Trader", "decision"],
        ["risk_aggressive", "Aggressive Analyst", "risk"],
        ["risk_conservative", "Conservative Analyst", "risk"],
        ["risk_neutral", "Neutral Analyst", "risk"],
        ["final_decision", "Portfolio Manager", "final"],
      ];

      const loaded: StepStatus[] = [];
      const expanded = new Set<string>();
      for (const [field, agent, phase] of fieldMap) {
        const content = detail[field];
        if (content) {
          loaded.push({ agent, phase, status: "done", content });
          expanded.add(`${phase}-${agent}`);
        }
      }
      setSteps(loaded);
      setExpandedCards(expanded);
    } catch (e) {
      setError(String(e));
    }
  };

  const toggleCard = (key: string) => {
    setExpandedCards((prev) => {
      const next = new Set(prev);
      if (next.has(key)) next.delete(key); else next.add(key);
      return next;
    });
  };

  const signalColor = (s: string) => {
    if (s === "BUY" || s === "OVERWEIGHT") return "bg-emerald-500";
    if (s === "SELL" || s === "UNDERWEIGHT") return "bg-red-500";
    return "bg-yellow-500";
  };

  const displaySteps = steps.length > 0
    ? steps
    : ALL_STEPS.map((s) => ({ ...s, status: "pending" as const }));

  return (
    <>
      <header className="flex items-center gap-2 border-b px-4 py-3">
        <SidebarTrigger />
        <Separator orientation="vertical" className="h-4" />
        <h1 className="text-sm font-semibold">{t("analysis.title")}</h1>
        <div className="ml-auto">
          <ThemeToggle />
        </div>
      </header>

      <div className="flex-1 flex gap-4 p-6 overflow-hidden">
        {/* Left: Config */}
        <div className="w-72 shrink-0 space-y-4 overflow-y-auto">
          <Card>
            <CardHeader>
              <CardTitle className="text-base">{t("analysis.config")}</CardTitle>
            </CardHeader>
            <CardContent className="space-y-3">
              <div className="space-y-1">
                <label className="text-xs text-muted-foreground">{t("analysis.symbol")}</label>
                <Input value={symbol} onChange={(e) => setSymbol(e.target.value.toUpperCase())} placeholder="2330.TW" />
              </div>
              <div className="space-y-1">
                <label className="text-xs text-muted-foreground">{t("analysis.debateRounds")}</label>
                <Input type="number" min={1} max={5} value={debateRounds} onChange={(e) => setDebateRounds(Number(e.target.value) || 1)} />
              </div>
              <div className="space-y-1">
                <label className="text-xs text-muted-foreground">{t("analysis.riskRounds")}</label>
                <Input type="number" min={1} max={5} value={riskRounds} onChange={(e) => setRiskRounds(Number(e.target.value) || 1)} />
              </div>
              <div className="space-y-1">
                <label className="text-xs text-muted-foreground">{t("analysis.cooldown")}</label>
                <Input type="number" min={0} max={60} value={cooldownSecs} onChange={(e) => setCooldownSecs(Number(e.target.value) || 0)} />
              </div>
              <Separator />
              <div className="space-y-2">
                <label className="text-xs text-muted-foreground">{t("analysis.analysts")}</label>
                {[
                  { label: t("analysis.marketAnalyst"), checked: enableMarket, set: setEnableMarket },
                  { label: t("analysis.newsAnalyst"), checked: enableNews, set: setEnableNews },
                  { label: t("analysis.fundamentalsAnalyst"), checked: enableFundamentals, set: setEnableFundamentals },
                  { label: t("analysis.socialAnalyst"), checked: enableSocial, set: setEnableSocial },
                ].map((item) => (
                  <div key={item.label} className="flex items-center justify-between">
                    <span className="text-sm">{item.label}</span>
                    <Switch checked={item.checked} onCheckedChange={item.set} />
                  </div>
                ))}
              </div>
              <Button className="w-full" onClick={handleRun} disabled={running}>
                {running ? t("analysis.running") : t("analysis.run")}
              </Button>
            </CardContent>
          </Card>

          {/* History */}
          {history.length > 0 && (
            <Card>
              <CardHeader>
                <CardTitle className="text-base">{t("analysis.history")}</CardTitle>
              </CardHeader>
              <CardContent className="space-y-2">
                {history.map((h) => (
                  <button
                    key={h.id}
                    className="flex w-full items-center gap-2 rounded-md border p-2 text-left text-sm hover:bg-muted/50"
                    onClick={() => loadHistory(h)}
                  >
                    <Badge className={signalColor(h.signal)} variant="default">
                      {h.signal}
                    </Badge>
                    <span className="flex-1 font-medium">{h.symbol}</span>
                    <span className="text-xs text-muted-foreground">
                      {h.created_at.split(" ")[0]}
                    </span>
                  </button>
                ))}
              </CardContent>
            </Card>
          )}
        </div>

        {/* Right: Phase-grouped grid */}
        <ScrollArea className="flex-1">
          <div className="space-y-6 pr-3">
            {/* Signal */}
            {signal && (
              <Card>
                <CardContent className="flex items-center justify-center py-6">
                  <Badge className={`text-2xl px-6 py-2 ${signalColor(signal)}`} variant="default">
                    {signal}
                  </Badge>
                </CardContent>
              </Card>
            )}

            {error && (
              <Card className="border-red-500">
                <CardContent className="pt-4 text-red-500 text-sm">{error}</CardContent>
              </Card>
            )}

            {/* Grouped by phase */}
            {(() => {
              const grouped = groupByPhase(displaySteps);
              return PHASE_ORDER.filter((phase) => grouped[phase]).map((phase) => (
                <div key={phase} className="space-y-2">
                  <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
                    {t(PHASE_LABELS[phase])}
                  </h3>
                  <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
                    {grouped[phase].map((step, i) => {
                      const key = stepKey(step);
                      const isOpen = expandedCards.has(key);

                      return (
                        <Card
                          key={`${key}-${i}`}
                          className={
                            step.status === "running" ? "ring-2 ring-primary/50 animate-pulse" :
                            step.status === "error" ? "ring-2 ring-destructive/50" : ""
                          }
                        >
                          <Collapsible open={isOpen} onOpenChange={() => toggleCard(key)}>
                            <CollapsibleTrigger asChild>
                              <CardHeader className="cursor-pointer py-3">
                                <div className="flex items-center gap-2">
                                  <span className={`text-sm ${step.status === "running" ? "animate-pulse" : ""}`}>
                                    {step.status === "running" ? "⏳" :
                                     step.status === "done" ? "✅" :
                                     step.status === "error" ? "❌" : "⬜"}
                                  </span>
                                  <span className="text-sm font-medium flex-1">{step.agent}</span>
                                  {step.statusMessage && step.status === "running" && (
                                    <span className="text-xs text-muted-foreground">{step.statusMessage}</span>
                                  )}
                                </div>
                              </CardHeader>
                            </CollapsibleTrigger>
                            <CollapsibleContent>
                              <CardContent className="pt-0">
                                {step.status === "running" && (
                                  <div className="space-y-2">
                                    <Skeleton className="h-4 w-full" />
                                    <Skeleton className="h-4 w-3/4" />
                                    <Skeleton className="h-4 w-5/6" />
                                  </div>
                                )}
                                {step.content && (
                                  <div className="max-h-60 overflow-y-auto prose prose-sm dark:prose-invert max-w-none">
                                    <Markdown>{step.content}</Markdown>
                                  </div>
                                )}
                                {step.status === "error" && step.statusMessage && (
                                  <p className="text-xs text-destructive">{step.statusMessage}</p>
                                )}
                              </CardContent>
                            </CollapsibleContent>
                          </Collapsible>
                        </Card>
                      );
                    })}
                  </div>
                </div>
              ));
            })()}
            <div ref={bottomRef} />
          </div>
        </ScrollArea>
      </div>
    </>
  );
}
