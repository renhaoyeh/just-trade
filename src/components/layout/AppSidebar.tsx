import {
  ChartLine,
  Wallet,
  ClockCounterClockwise,
  Binoculars,
  GearSix,
  SignOut,
  Lightning,
} from "@phosphor-icons/react";
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuBadge,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarSeparator,
} from "@/components/ui/sidebar";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { useTranslation } from "react-i18next";

const mainNav = [
  { titleKey: "nav.dashboard", icon: ChartLine, isActive: true },
  { titleKey: "nav.portfolio", icon: Wallet },
  { titleKey: "nav.trade", icon: Lightning, badgeKey: "nav.new" },
  { titleKey: "nav.history", icon: ClockCounterClockwise },
  { titleKey: "nav.watchlist", icon: Binoculars },
];

const bottomNav = [
  { titleKey: "nav.settings", icon: GearSix },
  { titleKey: "nav.signOut", icon: SignOut },
];

export function AppSidebar() {
  const { t } = useTranslation();

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
                    isActive={item.isActive}
                  >
                    <item.icon className="size-4" />
                    <span>{t(item.titleKey)}</span>
                  </SidebarMenuButton>
                  {item.badgeKey && (
                    <SidebarMenuBadge>{t(item.badgeKey)}</SidebarMenuBadge>
                  )}
                </SidebarMenuItem>
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>

      <SidebarFooter>
        <SidebarMenu>
          {bottomNav.map((item) => (
            <SidebarMenuItem key={item.titleKey}>
              <SidebarMenuButton tooltip={t(item.titleKey)}>
                <item.icon className="size-4" />
                <span>{t(item.titleKey)}</span>
              </SidebarMenuButton>
            </SidebarMenuItem>
          ))}
          <SidebarSeparator />
          <SidebarMenuItem>
            <SidebarMenuButton size="lg" tooltip={t("nav.user")}>
              <Avatar size="sm">
                <AvatarFallback>JT</AvatarFallback>
              </Avatar>
              <div className="flex flex-col gap-0.5 leading-none">
                <span className="font-medium">{t("nav.traderName")}</span>
                <span className="text-xs text-muted-foreground">{t("nav.traderEmail")}</span>
              </div>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarFooter>
    </Sidebar>
  );
}
