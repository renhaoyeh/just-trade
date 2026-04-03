import { useState } from "react";
import reactLogo from "@/assets/react.svg";
import { invoke } from "@tauri-apps/api/core";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");

  async function greet() {
    setGreetMsg(await invoke("greet", { name }));
  }

  return (
    <main className="m-0 flex flex-col items-center justify-center pt-[10vh] text-center">
      <h1 className="text-3xl font-bold">Welcome to Tauri + React</h1>

      <div className="mt-6 flex items-center justify-center gap-6">
        <a href="https://vite.dev" target="_blank">
          <img
            src="/vite.svg"
            className="h-24 p-4 transition-all duration-700 hover:drop-shadow-[0_0_2em_#747bff]"
            alt="Vite logo"
          />
        </a>
        <a href="https://tauri.app" target="_blank">
          <img
            src="/tauri.svg"
            className="h-24 p-4 transition-all duration-700 hover:drop-shadow-[0_0_2em_#24c8db]"
            alt="Tauri logo"
          />
        </a>
        <a href="https://react.dev" target="_blank">
          <img
            src={reactLogo}
            className="h-24 p-4 transition-all duration-700 hover:drop-shadow-[0_0_2em_#61dafb]"
            alt="React logo"
          />
        </a>
      </div>

      <p className="mt-4 text-muted-foreground">
        Click on the Tauri, Vite, and React logos to learn more.
      </p>

      <form
        className="mt-6 flex items-center justify-center gap-2"
        onSubmit={(e) => {
          e.preventDefault();
          greet();
        }}
      >
        <input
          id="greet-input"
          onChange={(e) => setName(e.currentTarget.value)}
          placeholder="Enter a name..."
          className="rounded-lg border border-input bg-background px-3 py-2 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
        />
        <button
          type="submit"
          className="rounded-lg bg-primary px-4 py-2 text-sm font-medium text-primary-foreground shadow-sm transition-colors hover:bg-primary/90"
        >
          Greet
        </button>
      </form>

      <p className="mt-4 text-foreground">{greetMsg}</p>
    </main>
  );
}

export default App;
