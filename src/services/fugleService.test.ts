import { describe, it, expect, vi, beforeEach } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import {
  fugleWsConnect,
  fugleWsDisconnect,
  fugleWsSubscribe,
  fugleWsUnsubscribe,
  fugleWsStatus,
} from "./stockService";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const mockedInvoke = vi.mocked(invoke);

describe("Fugle WebSocket service", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("fugleWsConnect", () => {
    it("calls invoke with correct command and api key", async () => {
      mockedInvoke.mockResolvedValue(undefined);
      await fugleWsConnect("test-api-key");
      expect(mockedInvoke).toHaveBeenCalledWith("fugle_ws_connect", {
        apiKey: "test-api-key",
      });
    });

    it("propagates errors", async () => {
      mockedInvoke.mockRejectedValue("Connection failed");
      await expect(fugleWsConnect("bad-key")).rejects.toBe("Connection failed");
    });
  });

  describe("fugleWsDisconnect", () => {
    it("calls invoke with correct command", async () => {
      mockedInvoke.mockResolvedValue(undefined);
      await fugleWsDisconnect();
      expect(mockedInvoke).toHaveBeenCalledWith("fugle_ws_disconnect");
    });
  });

  describe("fugleWsSubscribe", () => {
    it("calls invoke with channel and symbol", async () => {
      mockedInvoke.mockResolvedValue(undefined);
      await fugleWsSubscribe("trades", "2330");
      expect(mockedInvoke).toHaveBeenCalledWith("fugle_ws_subscribe", {
        channel: "trades",
        symbol: "2330",
      });
    });
  });

  describe("fugleWsUnsubscribe", () => {
    it("calls invoke with channel and symbol", async () => {
      mockedInvoke.mockResolvedValue(undefined);
      await fugleWsUnsubscribe("books", "2454");
      expect(mockedInvoke).toHaveBeenCalledWith("fugle_ws_unsubscribe", {
        channel: "books",
        symbol: "2454",
      });
    });
  });

  describe("fugleWsStatus", () => {
    it("returns connection status", async () => {
      mockedInvoke.mockResolvedValue(true);
      const result = await fugleWsStatus();
      expect(result).toBe(true);
      expect(mockedInvoke).toHaveBeenCalledWith("fugle_ws_status");
    });
  });
});
