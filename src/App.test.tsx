import { render, screen } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import App from "@/App";

describe("App", () => {
  it("renders the dashboard header", () => {
    render(<App />);
    const headings = screen.getAllByText("總覽");
    expect(headings.length).toBeGreaterThan(0);
  });

  it("renders the stock search input", () => {
    render(<App />);
    expect(
      screen.getByPlaceholderText("輸入股票代號 (如 2330)")
    ).toBeInTheDocument();
  });

  it("renders the default selected symbol", () => {
    render(<App />);
    const symbols = screen.getAllByText("2330.TW");
    expect(symbols.length).toBeGreaterThan(0);
  });

  it("renders tab triggers", () => {
    render(<App />);
    expect(screen.getByRole("tab", { name: "自選清單" })).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: "歷史股價" })).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: "相關新聞" })).toBeInTheDocument();
  });
});
