import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
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
  getLlmModels,
  type AppSettings,
} from "@/services/stockService";

const PROVIDERS = [
  { value: "openai", label: "OpenAI" },
  { value: "anthropic", label: "Anthropic" },
  { value: "google", label: "Google Gemini" },
  { value: "ollama", label: "Ollama (Local)" },
];

export default function Settings() {
  const { t } = useTranslation();
  const [settings, setSettings] = useState<AppSettings>({
    llm_provider: "openai",
    llm_model: null,
    openai_api_key: null,
    anthropic_api_key: null,
    google_api_key: null,
    ollama_base_url: null,
  });
  const [models, setModels] = useState<string[]>([]);
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    getSettings().then(setSettings).catch(console.error);
  }, []);

  useEffect(() => {
    if (settings.llm_provider) {
      getLlmModels(settings.llm_provider)
        .then(setModels)
        .catch(() => setModels([]));
    }
  }, [settings.llm_provider]);

  const update = (key: keyof AppSettings, value: string | null) => {
    setSettings((prev) => ({ ...prev, [key]: value }));
    setSaved(false);
  };

  const handleSave = async () => {
    setSaving(true);
    try {
      await saveSettings(settings);
      setSaved(true);
    } catch (e) {
      console.error(e);
    } finally {
      setSaving(false);
    }
  };

  const provider = settings.llm_provider || "openai";

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

      <div className="flex-1 space-y-6 p-6 max-w-2xl">
        {/* LLM Provider */}
        <Card>
          <CardHeader>
            <CardTitle>{t("settings.llmProvider")}</CardTitle>
            <CardDescription>{t("settings.llmProviderDesc")}</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <label className="text-sm font-medium">{t("settings.provider")}</label>
              <Select value={provider} onValueChange={(v) => update("llm_provider", v)}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {PROVIDERS.map((p) => (
                    <SelectItem key={p.value} value={p.value}>
                      {p.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div className="space-y-2">
              <label className="text-sm font-medium">{t("settings.model")}</label>
              {models.length > 0 ? (
                <Select
                  value={settings.llm_model || ""}
                  onValueChange={(v) => update("llm_model", v)}
                >
                  <SelectTrigger>
                    <SelectValue placeholder={t("settings.selectModel")} />
                  </SelectTrigger>
                  <SelectContent>
                    {models.map((m) => (
                      <SelectItem key={m} value={m}>
                        {m}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              ) : (
                <Input
                  value={settings.llm_model || ""}
                  onChange={(e) => update("llm_model", e.target.value || null)}
                  placeholder={t("settings.modelPlaceholder")}
                />
              )}
            </div>
          </CardContent>
        </Card>

        {/* API Keys */}
        <Card>
          <CardHeader>
            <CardTitle>{t("settings.apiKeys")}</CardTitle>
            <CardDescription>{t("settings.apiKeysDesc")}</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            {provider === "openai" && (
              <div className="space-y-2">
                <label className="text-sm font-medium">OpenAI API Key</label>
                <Input
                  type="password"
                  value={settings.openai_api_key || ""}
                  onChange={(e) => update("openai_api_key", e.target.value || null)}
                  placeholder="sk-..."
                />
              </div>
            )}
            {provider === "anthropic" && (
              <div className="space-y-2">
                <label className="text-sm font-medium">Anthropic API Key</label>
                <Input
                  type="password"
                  value={settings.anthropic_api_key || ""}
                  onChange={(e) => update("anthropic_api_key", e.target.value || null)}
                  placeholder="sk-ant-..."
                />
              </div>
            )}
            {provider === "google" && (
              <div className="space-y-2">
                <label className="text-sm font-medium">Google API Key</label>
                <Input
                  type="password"
                  value={settings.google_api_key || ""}
                  onChange={(e) => update("google_api_key", e.target.value || null)}
                  placeholder="AIza..."
                />
              </div>
            )}
            {provider === "ollama" && (
              <div className="space-y-2">
                <label className="text-sm font-medium">Ollama Base URL</label>
                <Input
                  value={settings.ollama_base_url || ""}
                  onChange={(e) => update("ollama_base_url", e.target.value || null)}
                  placeholder="http://localhost:11434"
                />
              </div>
            )}
          </CardContent>
        </Card>

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
