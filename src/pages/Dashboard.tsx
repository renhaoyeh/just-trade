import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { Separator } from "@/components/ui/separator";
import { Progress } from "@/components/ui/progress";

const portfolioData = [
  { symbol: "AAPL", name: "Apple Inc.", price: 198.45, change: +2.34, changePercent: +1.19, shares: 50, value: 9922.5 },
  { symbol: "TSLA", name: "Tesla Inc.", price: 248.12, change: -5.67, changePercent: -2.23, shares: 20, value: 4962.4 },
  { symbol: "NVDA", name: "NVIDIA Corp.", price: 875.30, change: +12.45, changePercent: +1.44, shares: 10, value: 8753.0 },
  { symbol: "MSFT", name: "Microsoft Corp.", price: 415.60, change: +3.21, changePercent: +0.78, shares: 15, value: 6234.0 },
  { symbol: "AMZN", name: "Amazon.com Inc.", price: 185.90, change: -1.12, changePercent: -0.60, shares: 30, value: 5577.0 },
];

const recentTrades = [
  { id: 1, symbol: "AAPL", side: "BUY", qty: 10, price: 196.11, time: "14:32:05", status: "filled" },
  { id: 2, symbol: "TSLA", side: "SELL", qty: 5, price: 253.79, time: "13:15:22", status: "filled" },
  { id: 3, symbol: "NVDA", side: "BUY", qty: 3, price: 862.85, time: "11:45:10", status: "filled" },
  { id: 4, symbol: "AMZN", side: "BUY", qty: 15, price: 187.02, time: "10:30:44", status: "partial" },
  { id: 5, symbol: "MSFT", side: "SELL", qty: 8, price: 412.39, time: "09:31:02", status: "pending" },
];

const watchlist = [
  { symbol: "META", price: 505.75, change: +3.82 },
  { symbol: "GOOG", price: 176.40, change: -0.95 },
  { symbol: "AMD", price: 162.55, change: +5.12 },
  { symbol: "NFLX", price: 628.90, change: +8.44 },
];

function StatCard({ title, value, description, trend }: {
  title: string;
  value: string;
  description: string;
  trend?: "up" | "down";
}) {
  return (
    <Card>
      <CardHeader>
        <CardDescription>{title}</CardDescription>
        <CardTitle className="text-2xl tabular-nums">
          {value}
        </CardTitle>
      </CardHeader>
      <CardContent>
        <span className={trend === "up" ? "text-emerald-500" : trend === "down" ? "text-red-500" : "text-muted-foreground"}>
          {description}
        </span>
      </CardContent>
    </Card>
  );
}

export default function Dashboard() {
  return (
    <div className="flex min-h-screen flex-col bg-background">
      {/* Header */}
      <header className="flex items-center justify-between border-b px-6 py-3">
        <div className="flex items-center gap-3">
          <h1 className="text-lg font-bold tracking-tight">Just Trade</h1>
          <Separator orientation="vertical" className="h-5" />
          <Badge variant="outline">Live</Badge>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="ghost" size="sm">Settings</Button>
          <Avatar size="sm">
            <AvatarFallback>JT</AvatarFallback>
          </Avatar>
        </div>
      </header>

      {/* Main content */}
      <main className="flex-1 space-y-6 p-6">
        {/* Stats row */}
        <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
          <StatCard
            title="Total Portfolio"
            value="$35,448.90"
            description="+$1,234.56 (+3.61%)"
            trend="up"
          />
          <StatCard
            title="Today's P&L"
            value="+$487.22"
            description="+1.39% from open"
            trend="up"
          />
          <StatCard
            title="Buying Power"
            value="$12,550.00"
            description="64% available"
          />
          <StatCard
            title="Open Orders"
            value="2"
            description="1 partial, 1 pending"
          />
        </div>

        {/* Tabs section */}
        <Tabs defaultValue="portfolio">
          <TabsList>
            <TabsTrigger value="portfolio">Portfolio</TabsTrigger>
            <TabsTrigger value="trades">Recent Trades</TabsTrigger>
            <TabsTrigger value="watchlist">Watchlist</TabsTrigger>
          </TabsList>

          {/* Portfolio tab */}
          <TabsContent value="portfolio">
            <Card>
              <CardHeader>
                <CardTitle>Holdings</CardTitle>
                <CardDescription>Your current positions</CardDescription>
              </CardHeader>
              <CardContent>
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>Symbol</TableHead>
                      <TableHead>Name</TableHead>
                      <TableHead className="text-right">Price</TableHead>
                      <TableHead className="text-right">Change</TableHead>
                      <TableHead className="text-right">Shares</TableHead>
                      <TableHead className="text-right">Value</TableHead>
                      <TableHead className="text-right">Allocation</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {portfolioData.map((stock) => {
                      const totalValue = portfolioData.reduce((sum, s) => sum + s.value, 0);
                      const allocation = (stock.value / totalValue) * 100;
                      return (
                        <TableRow key={stock.symbol}>
                          <TableCell className="font-medium">{stock.symbol}</TableCell>
                          <TableCell className="text-muted-foreground">{stock.name}</TableCell>
                          <TableCell className="text-right tabular-nums">${stock.price.toFixed(2)}</TableCell>
                          <TableCell className={`text-right tabular-nums ${stock.change >= 0 ? "text-emerald-500" : "text-red-500"}`}>
                            {stock.change >= 0 ? "+" : ""}{stock.change.toFixed(2)} ({stock.changePercent >= 0 ? "+" : ""}{stock.changePercent.toFixed(2)}%)
                          </TableCell>
                          <TableCell className="text-right tabular-nums">{stock.shares}</TableCell>
                          <TableCell className="text-right tabular-nums">${stock.value.toLocaleString()}</TableCell>
                          <TableCell className="text-right">
                            <div className="flex items-center justify-end gap-2">
                              <Progress value={allocation} className="w-16" />
                              <span className="w-10 tabular-nums text-muted-foreground">{allocation.toFixed(0)}%</span>
                            </div>
                          </TableCell>
                        </TableRow>
                      );
                    })}
                  </TableBody>
                </Table>
              </CardContent>
            </Card>
          </TabsContent>

          {/* Recent Trades tab */}
          <TabsContent value="trades">
            <Card>
              <CardHeader>
                <CardTitle>Recent Trades</CardTitle>
                <CardDescription>Today's executed orders</CardDescription>
              </CardHeader>
              <CardContent>
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>Time</TableHead>
                      <TableHead>Symbol</TableHead>
                      <TableHead>Side</TableHead>
                      <TableHead className="text-right">Qty</TableHead>
                      <TableHead className="text-right">Price</TableHead>
                      <TableHead className="text-right">Status</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {recentTrades.map((trade) => (
                      <TableRow key={trade.id}>
                        <TableCell className="tabular-nums text-muted-foreground">{trade.time}</TableCell>
                        <TableCell className="font-medium">{trade.symbol}</TableCell>
                        <TableCell>
                          <Badge variant={trade.side === "BUY" ? "default" : "secondary"}>
                            {trade.side}
                          </Badge>
                        </TableCell>
                        <TableCell className="text-right tabular-nums">{trade.qty}</TableCell>
                        <TableCell className="text-right tabular-nums">${trade.price.toFixed(2)}</TableCell>
                        <TableCell className="text-right">
                          <Badge
                            variant={
                              trade.status === "filled" ? "outline"
                                : trade.status === "partial" ? "secondary"
                                  : "destructive"
                            }
                          >
                            {trade.status}
                          </Badge>
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </CardContent>
            </Card>
          </TabsContent>

          {/* Watchlist tab */}
          <TabsContent value="watchlist">
            <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
              {watchlist.map((stock) => (
                <Card key={stock.symbol}>
                  <CardHeader>
                    <CardTitle className="flex items-center justify-between">
                      {stock.symbol}
                      <Badge variant={stock.change >= 0 ? "default" : "destructive"}>
                        {stock.change >= 0 ? "+" : ""}{stock.change.toFixed(2)}
                      </Badge>
                    </CardTitle>
                  </CardHeader>
                  <CardContent>
                    <div className="text-2xl font-bold tabular-nums">${stock.price.toFixed(2)}</div>
                  </CardContent>
                </Card>
              ))}
            </div>
          </TabsContent>
        </Tabs>
      </main>
    </div>
  );
}
