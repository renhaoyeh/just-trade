import { render, screen } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import App from "@/App";

describe("App", () => {
  it("renders the dashboard heading", () => {
    render(<App />);
    expect(screen.getByText("Just Trade")).toBeInTheDocument();
  });

  it("renders portfolio stat cards", () => {
    render(<App />);
    expect(screen.getByText("Total Portfolio")).toBeInTheDocument();
    expect(screen.getByText("$35,448.90")).toBeInTheDocument();
  });

  it("renders portfolio holdings table", () => {
    render(<App />);
    expect(screen.getByText("AAPL")).toBeInTheDocument();
    expect(screen.getByText("NVDA")).toBeInTheDocument();
  });
});
