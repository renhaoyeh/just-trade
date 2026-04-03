import { useState, useEffect } from "react";
import { useParams, useNavigate } from "react-router";
import { useTranslation } from "react-i18next";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  CardDescription,
} from "@/components/ui/card";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { SidebarTrigger } from "@/components/ui/sidebar";
import { ThemeToggle } from "@/components/common/ThemeToggle";
import { ArrowLeft } from "@phosphor-icons/react";
import type { SectorStock } from "@/types/stock";
import { getSectorStocks } from "@/services/stockService";

export default function SectorDetail() {
  const { t } = useTranslation();
  const { sectorName } = useParams<{ sectorName: string }>();
  const navigate = useNavigate();
  const [stocks, setStocks] = useState<SectorStock[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const sector = decodeURIComponent(sectorName || "");

  useEffect(() => {
    if (!sector) return;
    setLoading(true);
    getSectorStocks(sector)
      .then(setStocks)
      .catch((e) => setError(String(e)))
      .finally(() => setLoading(false));
  }, [sector]);

  return (
    <>
      <header className="flex items-center gap-2 border-b px-4 py-3">
        <SidebarTrigger />
        <Separator orientation="vertical" className="h-4" />
        <Button
          variant="ghost"
          size="sm"
          className="gap-1"
          onClick={() => navigate("/sectors")}
        >
          <ArrowLeft className="size-4" />
          {t("sectorDetail.backToSectors")}
        </Button>
        <Separator orientation="vertical" className="h-4" />
        <h1 className="text-sm font-semibold">{sector}</h1>
        <div className="ml-auto">
          <ThemeToggle />
        </div>
      </header>

      <div className="flex-1 space-y-6 p-6">
        {error && (
          <Card className="border-red-500">
            <CardContent className="pt-4 text-red-500">{error}</CardContent>
          </Card>
        )}

        <Card>
          <CardHeader>
            <CardTitle>{sector}</CardTitle>
            <CardDescription>
              {stocks.length > 0
                ? t("sectors.stockCount", { count: stocks.length })
                : ""}
            </CardDescription>
          </CardHeader>
          <CardContent>
            {loading ? (
              <p className="text-muted-foreground">{t("sectorDetail.loading")}</p>
            ) : stocks.length === 0 ? (
              <p className="text-muted-foreground">{t("sectorDetail.empty")}</p>
            ) : (
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>{t("sectorDetail.symbol")}</TableHead>
                    <TableHead>{t("sectorDetail.name")}</TableHead>
                    <TableHead className="text-right">{t("sectorDetail.price")}</TableHead>
                    <TableHead className="text-right">{t("sectorDetail.change")}</TableHead>
                    <TableHead className="text-right">{t("sectorDetail.volume")}</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {stocks.map((stock) => (
                    <TableRow
                      key={stock.symbol}
                      className="cursor-pointer hover:bg-muted/50"
                      onClick={() => navigate(`/?symbol=${stock.symbol}.TW`)}
                    >
                      <TableCell className="font-medium tabular-nums">
                        {stock.symbol}
                      </TableCell>
                      <TableCell>{stock.name}</TableCell>
                      <TableCell className="text-right tabular-nums">
                        {stock.closing_price?.toFixed(2) ?? "-"}
                      </TableCell>
                      <TableCell
                        className={`text-right tabular-nums ${
                          stock.change != null && stock.change > 0
                            ? "text-emerald-500"
                            : stock.change != null && stock.change < 0
                              ? "text-red-500"
                              : ""
                        }`}
                      >
                        {stock.change != null
                          ? `${stock.change > 0 ? "+" : ""}${stock.change.toFixed(2)}`
                          : "-"}
                      </TableCell>
                      <TableCell className="text-right tabular-nums">
                        {stock.trade_volume != null
                          ? `${(stock.trade_volume / 1000).toFixed(0)}K`
                          : "-"}
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            )}
          </CardContent>
        </Card>
      </div>
    </>
  );
}
