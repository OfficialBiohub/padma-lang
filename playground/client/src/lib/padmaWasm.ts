/* Style contract: the runtime stays invisible to the learner; only fast, localized results surface in the console. */

import { runPadma as fallbackRun, type PadmaRunResult } from "@/lib/padmaRunner";

let wasmPromise: Promise<(source: string) => string> | null = null;

async function loadWasm() {
  if (!wasmPromise) {
    const dynamicImport = new Function("path", "return import(path)") as (path: string) => Promise<{ default: () => Promise<unknown>; run_padma: (source: string) => string }>;
    wasmPromise = dynamicImport("/padma-wasm/padma_wasm.js").then(async (module) => {
      await module.default();
      return module.run_padma;
    });
  }
  return wasmPromise;
}

export async function runPadmaInBrowser(source: string): Promise<PadmaRunResult> {
  try {
    const run = await loadWasm();
    const response = run(source);
    const [status, ...rest] = response.split("\n");
    const locale = source.includes("let") && !source.includes("ধরি") ? "en" : "bn";
    if (status === "OK") {
      return { ok: true, output: rest.filter(Boolean), diagnostics: [], duration: "WASM", locale };
    }
    return { ok: false, output: [], diagnostics: [rest.join("\n") || "Padma returned an unknown diagnostic."], duration: "WASM", locale };
  } catch {
    return fallbackRun(source);
  }
}
