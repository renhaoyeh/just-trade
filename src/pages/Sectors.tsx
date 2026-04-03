import { useState, useEffect } from "react";
import { useNavigate } from "react-router";
import { useTranslation } from "react-i18next";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  CardDescription,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { SidebarTrigger } from "@/components/ui/sidebar";
import { ThemeToggle } from "@/components/common/ThemeToggle";
import type { Sector } from "@/types/stock";
import { getSectors } from "@/services/stockService";

export default function Sectors() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const [sectors, setSectors] = useState<Sector[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setLoading(true);
    getSectors()
      .then(setSectors)
      .catch((e) => setError(String(e)))
      .finally(() => setLoading(false));
  }, []);

  return (
    <>
      <header className="flex items-center gap-2 border-b px-4 py-3">
        <SidebarTrigger />
        <Separator orientation="vertical" className="h-4" />
        <h1 className="text-sm font-semibold">{t("sectors.title")}</h1>
        <div className="ml-auto">
          <ThemeToggle />
        </div>
      </header>

      <div className="flex-1 space-y-6 p-6">
        <CardDescription>{t("sectors.description")}</CardDescription>

        {error && (
          <Card className="border-red-500">
            <CardContent className="pt-4 text-red-500">{error}</CardContent>
          </Card>
        )}

        {loading ? (
          <p className="text-muted-foreground">{t("sectors.loading")}</p>
        ) : sectors.length === 0 ? (
          <p className="text-muted-foreground">{t("sectors.empty")}</p>
        ) : (
          <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
            {sectors.map((sector) => (
              <Card
                key={sector.name}
                className="cursor-pointer transition-colors hover:bg-muted/50"
                onClick={() =>
                  navigate(`/sectors/${encodeURIComponent(sector.name)}`)
                }
              >
                <CardHeader>
                  <CardTitle className="flex items-center justify-between text-base">
                    {sector.name}
                    <Badge variant="outline">
                      {t("sectors.stockCount", { count: sector.stock_count })}
                    </Badge>
                  </CardTitle>
                </CardHeader>
              </Card>
            ))}
          </div>
        )}
      </div>
    </>
  );
}
