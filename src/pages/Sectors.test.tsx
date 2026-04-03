import { screen } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { renderWithProviders } from "@/tests/render-helpers";
import Sectors from "./Sectors";

const mockedInvoke = vi.mocked(invoke);

function renderSectors() {
  return renderWithProviders(<Sectors />);
}

describe("Sectors", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders the page title", () => {
    renderSectors();
    expect(screen.getByText("產業類股")).toBeInTheDocument();
  });

  it("shows loading state initially", () => {
    mockedInvoke.mockReturnValue(new Promise(() => {})); // never resolves
    renderSectors();
    expect(screen.getByText("載入產業資料中...")).toBeInTheDocument();
  });

  it("shows empty state when no sectors", async () => {
    mockedInvoke.mockResolvedValue([]);
    renderSectors();
    expect(await screen.findByText("無產業資料")).toBeInTheDocument();
  });

  it("renders sector cards from data", async () => {
    mockedInvoke.mockResolvedValue([
      { name: "半導體業", stock_count: 40 },
      { name: "金融保險業", stock_count: 30 },
    ]);
    renderSectors();
    expect(await screen.findByText("半導體業")).toBeInTheDocument();
    expect(screen.getByText("金融保險業")).toBeInTheDocument();
    expect(screen.getByText("40 檔")).toBeInTheDocument();
    expect(screen.getByText("30 檔")).toBeInTheDocument();
  });
});
