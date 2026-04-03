import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { renderWithProviders } from "@/tests/render-helpers";
import RealtimeQuotes from "./RealtimeQuotes";

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

const mockedInvoke = vi.mocked(invoke);

describe("RealtimeQuotes", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders page title", () => {
    mockedInvoke.mockResolvedValue({});
    renderWithProviders(<RealtimeQuotes />);
    expect(screen.getByText("即時報價")).toBeInTheDocument();
  });

  it("shows no-api-key warning when Fugle key is not set", () => {
    mockedInvoke.mockResolvedValue({});
    renderWithProviders(<RealtimeQuotes />);
    expect(
      screen.getByText("請先至設定頁面填入 Fugle API Key")
    ).toBeInTheDocument();
  });

  it("shows connect button when API key is loaded", async () => {
    mockedInvoke.mockResolvedValue({ fugle: { api_key: "test-key" } });
    renderWithProviders(<RealtimeQuotes />);
    const btn = await screen.findByRole("button", { name: "連線" });
    expect(btn).toBeInTheDocument();
  });

  it("shows disconnected badge by default", () => {
    mockedInvoke.mockResolvedValue({});
    renderWithProviders(<RealtimeQuotes />);
    expect(screen.getByText("未連線")).toBeInTheDocument();
  });
});
