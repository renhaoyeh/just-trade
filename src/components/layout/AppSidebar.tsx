import {
  ChartLine,
  Lightning,
  Buildings,
  Robot,
  GearSix,
  Pulse,
  ChartBar,
} from "@phosphor-icons/react";
import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarSeparator,
} from "@/components/ui/sidebar";
import { useTranslation } from "react-i18next";
import { useNavigate, useLocation } from "react-router";

const mainNav = [
  { titleKey: "nav.dashboard", icon: ChartLine, path: "/" },
  { titleKey: "nav.sectors", icon: Buildings, path: "/sectors" },
  { titleKey: "nav.tradingAgents", icon: Robot, path: "/trading-agents" },
  { titleKey: "nav.backtest", icon: ChartBar, path: "/backtest" },
  { titleKey: "nav.realtime", icon: Pulse, path: "/realtime" },
  { titleKey: "nav.settings", icon: GearSix, path: "/settings" },
];

export function AppSidebar() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const location = useLocation();

  return (
    <Sidebar collapsible="icon">
      <SidebarHeader>
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton size="lg" tooltip={t("app.name")}>
              <div className="flex size-8 items-center justify-center rounded-md bg-primary text-primary-foreground">
                <Lightning className="size-4" weight="bold" />
              </div>
              <div className="flex flex-col gap-0.5 leading-none">
                <span className="font-semibold">{t("app.name")}</span>
                <span className="text-xs text-muted-foreground">{t("app.version")}</span>
              </div>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarHeader>

      <SidebarSeparator />

      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupLabel>{t("nav.navigation")}</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {mainNav.map((item) => (
                <SidebarMenuItem key={item.titleKey}>
                  <SidebarMenuButton
                    tooltip={t(item.titleKey)}
                    isActive={
                      item.path === "/"
                        ? location.pathname === "/"
                        : location.pathname.startsWith(item.path)
                    }
                    onClick={() => navigate(item.path)}
                  >
                    <item.icon className="size-4" />
                    <span>{t(item.titleKey)}</span>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>

    </Sidebar>
  );
}
