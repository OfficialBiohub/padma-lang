declare module "/padma-wasm/padma_wasm.js" {
  export default function init(): Promise<unknown>;
  export function run_padma(source: string): string;
}
