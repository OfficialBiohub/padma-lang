/* Style contract: execution copy mirrors the Night River Console direction—clear, calm, and actionable. */

export type PadmaRunResult = {
  ok: boolean;
  output: string[];
  diagnostics: string[];
  duration: string;
  locale: "bn" | "en";
};

const bnWords = ["ধরি", "দেখাও", "যদি", "নইলে", "সত্য", "মিথ্যা"];
const enWords = ["let", "print", "if", "else", "true", "false"];

function localeFor(source: string): "bn" | "en" {
  if (source.includes("padma:locale=en")) return "en";
  if (source.includes("padma:locale=bn")) return "bn";
  const bn = bnWords.reduce((sum, word) => sum + (source.match(new RegExp(word, "g")) ?? []).length, 0);
  const en = enWords.reduce((sum, word) => sum + (source.match(new RegExp(`\\b${word}\\b`, "g")) ?? []).length, 0);
  return en > bn ? "en" : "bn";
}

function normalizeDigits(value: string) {
  return value.replace(/[০-৯]/g, (digit) => String("০১২৩৪৫৬৭৮৯".indexOf(digit)));
}

function interpolate(value: string, vars: Map<string, string>) {
  return value.replace(/\{([^{}\s]+)\}/g, (_, key: string) => vars.get(key) ?? `{${key}}`);
}

function expression(raw: string, vars: Map<string, string>): string {
  const source = normalizeDigits(raw.trim());
  if (source === "সত্য" || source === "true") return "true";
  if (source === "মিথ্যা" || source === "false") return "false";
  if (/^"[\s\S]*"$/.test(source)) return interpolate(source.slice(1, -1), vars);
  if (/^[A-Za-z_\u0980-\u09FF][A-Za-z0-9_\u0980-\u09FF]*$/.test(source)) return vars.get(source) ?? "";

  const comparison = source.match(/^(.+?)\s*(===|==|!=|>=|<=|>|<)\s*(.+)$/);
  if (comparison) {
    const left = Number(expression(comparison[1], vars));
    const right = Number(expression(comparison[3], vars));
    const op = comparison[2];
    const result = op === "==" || op === "===" ? left === right : op === "!=" ? left !== right : op === ">" ? left > right : op === "<" ? left < right : op === ">=" ? left >= right : left <= right;
    return String(result);
  }

  const tokens = source.match(/\d+(?:\.\d+)?|[()+\-*/]|[A-Za-z_\u0980-\u09FF][A-Za-z0-9_\u0980-\u09FF]*/g) ?? [];
  if (!tokens.length) return source;
  const values = tokens.map((token) => /^[A-Za-z_\u0980-\u09FF]/.test(token) ? Number(vars.get(token) ?? 0) : Number(token));
  let total = values[0] ?? 0;
  for (let index = 1; index < tokens.length; index += 2) {
    const operator = tokens[index];
    const next = values[index + 1] ?? 0;
    if (operator === "+") total += next;
    if (operator === "-") total -= next;
    if (operator === "*") total *= next;
    if (operator === "/") {
      if (next === 0) throw new Error("P1011");
      total /= next;
    }
  }
  return Number.isInteger(total) ? String(total) : total.toFixed(4).replace(/0+$/, "").replace(/\.$/, "");
}

export function runPadma(source: string): PadmaRunResult {
  const started = performance.now();
  const locale = localeFor(source);
  const output: string[] = [];
  const diagnostics: string[] = [];
  const vars = new Map<string, string>();
  const lines = source.split(/\r?\n/);

  try {
    for (let index = 0; index < lines.length; index += 1) {
      const raw = lines[index].trim();
      if (!raw || raw.startsWith("#")) continue;
      const declaration = raw.match(/^(?:ধরি|let)\s+([A-Za-z_\u0980-\u09FF][A-Za-z0-9_\u0980-\u09FF]*)\s*=\s*(.+)$/);
      if (declaration) {
        const value = expression(declaration[2], vars);
        vars.set(declaration[1], value);
        continue;
      }
      const print = raw.match(/^(?:দেখাও|print)\s+(.+)$/);
      if (print) {
        const value = expression(print[1], vars);
        output.push(value);
        continue;
      }
      const ifLine = raw.match(/^(?:যদি|if)\s+(.+)\s*\{$/);
      if (ifLine) {
        const condition = expression(ifLine[1], vars) === "true";
        let depth = 1;
        let cursor = index + 1;
        const body: string[] = [];
        while (cursor < lines.length && depth > 0) {
          const current = lines[cursor].trim();
          if (current.endsWith("{")) depth += 1;
          if (current === "}") depth -= 1;
          if (depth > 0) body.push(lines[cursor]);
          cursor += 1;
        }
        if (condition) {
          const nested = runPadma(body.join("\n"));
          output.push(...nested.output);
        }
        index = cursor - 1;
        continue;
      }
      if (raw === "}" || /^(?:নইলে|else)\s*\{$/.test(raw)) continue;
      if (/^(?:ধরি|let|দেখাও|print|যদি|if)\b/.test(raw)) throw new Error(`P1003:${index + 1}`);
      throw new Error(`P1004:${index + 1}`);
    }
  } catch (caught) {
    const error = String(caught);
    const [code, line] = error.split(":");
    const lineNumber = line ?? "1";
    if (code === "P1011") {
      diagnostics.push(locale === "bn" ? `ত্রুটি[P1011] · লাইন ${lineNumber}: শূন্য দিয়ে ভাগ করা যাবে না।` : `error[P1011] · line ${lineNumber}: Cannot divide by zero.`);
    } else if (code === "P1003") {
      diagnostics.push(locale === "bn" ? `ত্রুটি[P1003] · লাইন ${lineNumber}: statement বা বন্ধনী পরীক্ষা করুন।` : `error[P1003] · line ${lineNumber}: Check the statement or delimiter.`);
    } else {
      diagnostics.push(locale === "bn" ? `ত্রুটি[P1004] · লাইন ${lineNumber}: এই statement Padma বুঝতে পারেনি।` : `error[P1004] · line ${lineNumber}: Padma could not understand this statement.`);
    }
  }

  const duration = `${Math.max(1, Math.round(performance.now() - started))} ms`;
  return { ok: diagnostics.length === 0, output, diagnostics, duration, locale };
}
