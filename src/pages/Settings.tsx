import { useState, useEffect, useCallback } from "react";
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
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  getSettings,
  saveSettings,
  testLlmConnection,
  type LlmConfig,
} from "@/services/stockService";

interface ProviderSettings {
  api_key: string | null;
  model: string | null;
  base_url: string | null;
}

interface AppSettings {
  openai: ProviderSettings | null;
  anthropic: ProviderSettings | null;
  google: ProviderSettings | null;
  groq: ProviderSettings | null;
  ollama: ProviderSettings | null;
}

const PROVIDERS = [
  { id: "openai", label: "OpenAI", placeholder: "sk-...", needsKey: true },
  { id: "anthropic", label: "Anthropic", placeholder: "sk-ant-...", needsKey: true },
  { id: "google", label: "Google Gemini", placeholder: "AIza...", needsKey: true },
  { id: "groq", label: "Groq", placeholder: "gsk_...", needsKey: true },
  { id: "ollama", label: "Ollama (Local)", placeholder: "http://localhost:11434", needsKey: false },
] as const;

type ProviderId = (typeof PROVIDERS)[number]["id"];

const empty = (): ProviderSettings => ({ api_key: null, model: null, base_url: null });

function ProviderCard({
  provider,
  settings,
  onChange,
}: {
  provider: (typeof PROVIDERS)[number];
  settings: ProviderSettings;
  onChange: (s: ProviderSettings) => void;
}) {
  const { t } = useTranslation();
  const [models, setModels] = useState<string[]>([]);
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<{ ok: boolean; msg: string } | null>(null);

  const handleTest = useCallback(async () => {
    setTesting(true);
    setTestResult(null);
    try {
      const config: LlmConfig = {
        provider: provider.id,
        model: settings.model || "",
        api_key: settings.api_key,
        base_url: provider.id === "ollama" ? (settings.base_url || "http://localhost:11434") : null,
      };
      const result = await testLlmConnection(config);
      setModels(result);
      setTestResult({ ok: true, msg: t("settings.testSuccess", { count: result.length }) });
    } catch (e) {
      setTestResult({ ok: false, msg: String(e) });
    } finally {
      setTesting(false);
    }
  }, [provider.id, settings, t]);

  const hasKey = provider.needsKey ? !!settings.api_key : true;

  // Auto-fetch models on mount if credentials exist
  const hasCredentials = provider.needsKey ? !!settings.api_key : !!settings.base_url;
  useEffect(() => {
    if (hasCredentials && models.length === 0 && !testing) {
      handleTest();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center justify-between text-base">
          {provider.label}
          {testResult && (
            <Badge variant={testResult.ok ? "default" : "destructive"}>
              {testResult.ok ? t("settings.connected") : t("settings.failed")}
            </Badge>
          )}
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        {/* API Key or Base URL */}
        {provider.needsKey ? (
          <div className="space-y-1">
            <label className="text-xs text-muted-foreground">API Key</label>
            <Input
              type="password"
              value={settings.api_key || ""}
              onChange={(e) => onChange({ ...settings, api_key: e.target.value || null })}
              placeholder={provider.placeholder}
            />
          </div>
        ) : (
          <div className="space-y-1">
            <label className="text-xs text-muted-foreground">{t("settings.serverUrl")}</label>
            <Input
              value={settings.base_url || ""}
              onChange={(e) => onChange({ ...settings, base_url: e.target.value || null })}
              placeholder={provider.placeholder}
            />
          </div>
        )}

        {/* Test + Model row */}
        <div className="flex items-center gap-2">
          <Button
            size="sm"
            variant="outline"
            onClick={handleTest}
            disabled={testing || !hasKey}
          >
            {testing ? t("settings.testing") : t("settings.test")}
          </Button>

          {models.length > 0 ? (
            <Select
              value={settings.model || ""}
              onValueChange={(v) => onChange({ ...settings, model: v })}
            >
              <SelectTrigger className="flex-1">
                <SelectValue placeholder={t("settings.selectModel")} />
              </SelectTrigger>
              <SelectContent>
                {models.map((m) => (
                  <SelectItem key={m} value={m}>{m}</SelectItem>
                ))}
              </SelectContent>
            </Select>
          ) : (
            <span className="text-xs text-muted-foreground">
              {t("settings.testFirst")}
            </span>
          )}
        </div>

        {/* Error message */}
        {testResult && !testResult.ok && (
          <p className="text-xs text-red-500">{testResult.msg}</p>
        )}
      </CardContent>
    </Card>
  );
}

export default function Settings() {
  const { t } = useTranslation();
  const [settings, setSettings] = useState<AppSettings>({
    openai: null,
    anthropic: null,
    google: null,
    groq: null,
    ollama: null,
  });
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    getSettings()
      .then((s) => setSettings(s as unknown as AppSettings))
      .catch(console.error);
  }, []);

  const updateProvider = (id: ProviderId, ps: ProviderSettings) => {
    setSettings((prev) => ({ ...prev, [id]: ps }));
    setSaved(false);
  };

  const handleSave = async () => {
    setSaving(true);
    try {
      await saveSettings(settings as never);
      setSaved(true);
    } catch (e) {
      console.error(e);
    } finally {
      setSaving(false);
    }
  };

  return (
    <>
      <header className="flex items-center gap-2 border-b px-4 py-3">
        <SidebarTrigger />
        <Separator orientation="vertical" className="h-4" />
        <h1 className="text-sm font-semibold">{t("settings.title")}</h1>
        <div className="ml-auto">
          <ThemeToggle />
        </div>
      </header>

      <div className="flex-1 space-y-4 p-6 max-w-2xl">
        {PROVIDERS.map((p) => (
          <ProviderCard
            key={p.id}
            provider={p}
            settings={settings[p.id] || empty()}
            onChange={(ps) => updateProvider(p.id, ps)}
          />
        ))}

        <div className="flex items-center gap-3">
          <Button onClick={handleSave} disabled={saving}>
            {saving ? t("settings.saving") : t("settings.save")}
          </Button>
          {saved && (
            <span className="text-sm text-emerald-500">{t("settings.saved")}</span>
          )}
        </div>
      </div>
    </>
  );
}
