import { useState, useEffect } from "react";
import Markdown from "react-markdown";
import { useTranslation } from "react-i18next";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { SidebarTrigger } from "@/components/ui/sidebar";
import { ThemeToggle } from "@/components/common/ThemeToggle";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Skeleton } from "@/components/ui/skeleton";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import { Button } from "@/components/ui/button";
import { ArrowLeft } from "@phosphor-icons/react";
import {
  getAnalysisHistory,
  getAnalysisDetail,
  type AnalysisRecord,
} from "@/services/stockService";

interface StepView {
  agent: string;
  phase: string;
  content: string;
}

const FIELD_MAP: [string, string, string][] = [
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

const PHASE_ORDER = ["analysts", "debate", "decision", "risk", "final"];
const PHASE_LABELS: Record<string, string> = {
  analysts: "analysis.phaseAnalysts",
  debate: "analysis.phaseDebate",
  decision: "analysis.phaseDecision",
  risk: "analysis.phaseRisk",
  final: "analysis.phaseFinal",
};

function signalColor(s: string) {
  if (s === "BUY" || s === "OVERWEIGHT") return "bg-emerald-500";
  if (s === "SELL" || s === "UNDERWEIGHT") return "bg-red-500";
  return "bg-yellow-500";
}

export default function AnalysisHistory() {
  const { t } = useTranslation();
  const [history, setHistory] = useState<AnalysisRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [selected, setSelected] = useState<AnalysisRecord | null>(null);
  const [steps, setSteps] = useState<StepView[]>([]);
  const [loadingDetail, setLoadingDetail] = useState(false);
  const [expandedCards, setExpandedCards] = useState<Set<string>>(new Set());

  useEffect(() => {
    setLoading(true);
    getAnalysisHistory(50)
      .then(setHistory)
      .catch(console.error)
      .finally(() => setLoading(false));
  }, []);

  const loadDetail = async (record: AnalysisRecord) => {
    setSelected(record);
    setLoadingDetail(true);
    try {
      const detail = (await getAnalysisDetail(record.id)) as Record<string, string>;
      const loaded: StepView[] = [];
      const expanded = new Set<string>();
      for (const [field, agent, phase] of FIELD_MAP) {
        const content = detail[field];
        if (content) {
          loaded.push({ agent, phase, content });
          expanded.add(`${phase}-${agent}`);
        }
      }
      setSteps(loaded);
      setExpandedCards(expanded);
    } catch (e) {
      console.error(e);
    } finally {
      setLoadingDetail(false);
    }
  };

  const toggleCard = (key: string) => {
    setExpandedCards((prev) => {
      const next = new Set(prev);
      if (next.has(key)) next.delete(key); else next.add(key);
      return next;
    });
  };

  const groupByPhase = (items: StepView[]) => {
    const groups: Record<string, StepView[]> = {};
    for (const item of items) {
      if (!groups[item.phase]) groups[item.phase] = [];
      groups[item.phase].push(item);
    }
    return groups;
  };

  // Detail view
  if (selected) {
    const grouped = groupByPhase(steps);
    return (
      <>
        <header className="flex items-center gap-2 border-b px-4 py-3">
          <SidebarTrigger />
          <Separator orientation="vertical" className="h-4" />
          <Button variant="ghost" size="sm" className="gap-1" onClick={() => setSelected(null)}>
            <ArrowLeft className="size-4" />
            {t("analysisHistory.backToList")}
          </Button>
          <Separator orientation="vertical" className="h-4" />
          <h1 className="text-sm font-semibold">{selected.symbol}</h1>
          <Badge className={signalColor(selected.signal)} variant="default">
            {selected.signal}
          </Badge>
          <span className="text-xs text-muted-foreground">{selected.created_at}</span>
          <div className="ml-auto">
            <ThemeToggle />
          </div>
        </header>

        <ScrollArea className="flex-1 p-6">
          {loadingDetail ? (
            <div className="space-y-3">
              <Skeleton className="h-24 w-full" />
              <Skeleton className="h-24 w-full" />
            </div>
          ) : (
            <div className="space-y-6">
              {PHASE_ORDER.filter((p) => grouped[p]).map((phase) => (
                <div key={phase} className="space-y-2">
                  <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
                    {t(PHASE_LABELS[phase])}
                  </h3>
                  <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
                    {grouped[phase].map((step) => {
                      const key = `${step.phase}-${step.agent}`;
                      const isOpen = expandedCards.has(key);
                      return (
                        <Card key={key}>
                          <Collapsible open={isOpen} onOpenChange={() => toggleCard(key)}>
                            <CollapsibleTrigger asChild>
                              <CardHeader className="cursor-pointer py-3">
                                <div className="flex items-center gap-2">
                                  <span className="text-sm">✅</span>
                                  <span className="text-sm font-medium flex-1">{step.agent}</span>
                                </div>
                              </CardHeader>
                            </CollapsibleTrigger>
                            <CollapsibleContent>
                              <CardContent className="pt-0">
                                <div className="max-h-60 overflow-y-auto prose prose-sm dark:prose-invert max-w-none">
                                  <Markdown>{step.content}</Markdown>
                                </div>
                              </CardContent>
                            </CollapsibleContent>
                          </Collapsible>
                        </Card>
                      );
                    })}
                  </div>
                </div>
              ))}
            </div>
          )}
        </ScrollArea>
      </>
    );
  }

  // List view
  return (
    <>
      <header className="flex items-center gap-2 border-b px-4 py-3">
        <SidebarTrigger />
        <Separator orientation="vertical" className="h-4" />
        <h1 className="text-sm font-semibold">{t("analysisHistory.title")}</h1>
        <div className="ml-auto">
          <ThemeToggle />
        </div>
      </header>

      <div className="flex-1 p-6">
        {loading ? (
          <div className="space-y-3">
            <Skeleton className="h-16 w-full" />
            <Skeleton className="h-16 w-full" />
            <Skeleton className="h-16 w-full" />
          </div>
        ) : history.length === 0 ? (
          <p className="text-muted-foreground">{t("analysisHistory.empty")}</p>
        ) : (
          <div className="space-y-2">
            {history.map((h) => (
              <Card
                key={h.id}
                className="cursor-pointer transition-colors hover:bg-muted/50"
                onClick={() => loadDetail(h)}
              >
                <CardHeader className="py-3">
                  <div className="flex items-center gap-3">
                    <Badge className={signalColor(h.signal)} variant="default">
                      {h.signal}
                    </Badge>
                    <CardTitle className="text-base flex-1">{h.symbol}</CardTitle>
                    <span className="text-sm text-muted-foreground">{h.created_at}</span>
                  </div>
                </CardHeader>
              </Card>
            ))}
          </div>
        )}
      </div>
    </>
  );
}
