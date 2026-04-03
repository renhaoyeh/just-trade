import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { listen } from "@tauri-apps/api/event";
import {
  Card,
  CardContent,
  CardDescription,
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
import {
  runAnalysis,
  getSettings,
  type PipelineConfig,
  type LlmConfig,
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
  message?: string;
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

export default function Analysis() {
  const { t } = useTranslation();
  const [symbol, setSymbol] = useState("2330.TW");
  const [running, setRunning] = useState(false);
  const [steps, setSteps] = useState<StepStatus[]>([]);
  const [signal, setSignal] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  // Config
  const [debateRounds, setDebateRounds] = useState(1);
  const [riskRounds, setRiskRounds] = useState(1);
  const [enableMarket, setEnableMarket] = useState(true);
  const [enableNews, setEnableNews] = useState(true);
  const [enableFundamentals, setEnableFundamentals] = useState(true);
  const [enableSocial, setEnableSocial] = useState(false);

  // Listen to progress events
  useEffect(() => {
    const unlisten = listen<ProgressEvent>("analysis-progress", (event) => {
      const { phase, agent, status, message } = event.payload;

      if (phase === "complete") {
        setSignal(message || "HOLD");
        return;
      }

      setSteps((prev) => {
        const existing = prev.find((s) => s.agent === agent && s.phase === phase);
        if (existing) {
          return prev.map((s) =>
            s.agent === agent && s.phase === phase
              ? { ...s, status: status as StepStatus["status"], message: message || undefined }
              : s
          );
        }
        return [...prev, { agent, phase, status: status as StepStatus["status"], message: message || undefined }];
      });
    });

    return () => { unlisten.then((fn) => fn()); };
  }, []);

  const handleRun = useCallback(async () => {
    setRunning(true);
    setSteps([]);
    setSignal(null);
    setError(null);

    // Build LLM config from saved settings
    let quickLlm: LlmConfig = { provider: "openai", model: "gpt-4o-mini" };
    let deepLlm: LlmConfig = { provider: "openai", model: "gpt-4o" };

    try {
      const settings = await getSettings();
      // Use first provider that has a key + model
      for (const [provider, cfg] of Object.entries(settings)) {
        if (cfg && cfg.api_key && cfg.model) {
          const base: LlmConfig = {
            provider,
            model: cfg.model,
            api_key: cfg.api_key,
            base_url: cfg.base_url,
          };
          quickLlm = base;
          deepLlm = { ...base };
          break;
        }
        // Ollama: no key needed
        if (provider === "ollama" && cfg && cfg.base_url && cfg.model) {
          const base: LlmConfig = {
            provider: "ollama",
            model: cfg.model,
            base_url: cfg.base_url,
          };
          quickLlm = base;
          deepLlm = { ...base };
          break;
        }
      }
    } catch {
      // Use defaults
    }

    const config: PipelineConfig = {
      quick_llm: quickLlm,
      deep_llm: deepLlm,
      max_debate_rounds: debateRounds,
      max_risk_rounds: riskRounds,
      enable_market_analyst: enableMarket,
      enable_news_analyst: enableNews,
      enable_fundamentals_analyst: enableFundamentals,
      enable_social_analyst: enableSocial,
    };

    try {
      await runAnalysis(symbol, config);
    } catch (e) {
      setError(String(e));
    } finally {
      setRunning(false);
    }
  }, [symbol, debateRounds, riskRounds, enableMarket, enableNews, enableFundamentals, enableSocial]);

  const statusIcon = (status: StepStatus["status"]) => {
    switch (status) {
      case "running": return "⏳";
      case "done": return "✅";
      case "error": return "❌";
      default: return "⬜";
    }
  };

  const signalColor = (s: string) => {
    if (s === "BUY" || s === "OVERWEIGHT") return "text-emerald-500";
    if (s === "SELL" || s === "UNDERWEIGHT") return "text-red-500";
    return "text-yellow-500";
  };

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

      <div className="flex-1 space-y-4 p-6">
        <div className="flex gap-4">
          {/* Left: Config + Run */}
          <div className="w-80 space-y-4">
            <Card>
              <CardHeader>
                <CardTitle className="text-base">{t("analysis.config")}</CardTitle>
              </CardHeader>
              <CardContent className="space-y-3">
                <div className="space-y-1">
                  <label className="text-xs text-muted-foreground">{t("analysis.symbol")}</label>
                  <Input
                    value={symbol}
                    onChange={(e) => setSymbol(e.target.value.toUpperCase())}
                    placeholder="2330.TW"
                  />
                </div>

                <div className="space-y-1">
                  <label className="text-xs text-muted-foreground">{t("analysis.debateRounds")}</label>
                  <Input
                    type="number" min={1} max={5}
                    value={debateRounds}
                    onChange={(e) => setDebateRounds(Number(e.target.value) || 1)}
                  />
                </div>

                <div className="space-y-1">
                  <label className="text-xs text-muted-foreground">{t("analysis.riskRounds")}</label>
                  <Input
                    type="number" min={1} max={5}
                    value={riskRounds}
                    onChange={(e) => setRiskRounds(Number(e.target.value) || 1)}
                  />
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
          </div>

          {/* Right: Progress */}
          <div className="flex-1 space-y-4">
            {/* Signal */}
            {signal && (
              <Card>
                <CardContent className="flex items-center justify-center py-6">
                  <span className={`text-4xl font-bold ${signalColor(signal)}`}>
                    {signal}
                  </span>
                </CardContent>
              </Card>
            )}

            {error && (
              <Card className="border-red-500">
                <CardContent className="pt-4 text-red-500 text-sm">{error}</CardContent>
              </Card>
            )}

            {/* Progress Steps */}
            <Card>
              <CardHeader>
                <CardTitle className="text-base">{t("analysis.progress")}</CardTitle>
                <CardDescription>
                  {steps.filter((s) => s.status === "done").length} / {steps.length || ALL_STEPS.length}
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-1">
                  {(steps.length > 0 ? steps : ALL_STEPS.map((s) => ({ ...s, status: "pending" as const }))).map((step, i) => (
                    <div key={`${step.phase}-${step.agent}-${i}`} className="flex items-center gap-2 py-1">
                      <span className="text-sm">{statusIcon(step.status)}</span>
                      <span className="text-sm flex-1">{step.agent}</span>
                      <Badge variant="outline" className="text-xs">{step.phase}</Badge>
                      {step.message && (
                        <span className="text-xs text-muted-foreground">{step.message}</span>
                      )}
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>
          </div>
        </div>
      </div>
    </>
  );
}
