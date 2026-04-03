import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import App from "./App";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((_cmd: string, args: { name: string }) =>
    Promise.resolve(`Hello, ${args.name}! You've been greeted from Rust!`)
  ),
}));

describe("App", () => {
  it("renders the welcome heading", () => {
    render(<App />);
    expect(
      screen.getByText("Welcome to Tauri + React")
    ).toBeInTheDocument();
  });

  it("greets the user when form is submitted", async () => {
    render(<App />);

    const input = screen.getByPlaceholderText("Enter a name...");
    const button = screen.getByText("Greet");

    fireEvent.change(input, { target: { value: "World" } });
    fireEvent.click(button);

    await waitFor(() => {
      expect(
        screen.getByText("Hello, World! You've been greeted from Rust!")
      ).toBeInTheDocument();
    });
  });
});
