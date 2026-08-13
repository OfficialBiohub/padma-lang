/* Style contract: Night River Console—code-first, indigo workbench, Padma teal focus, coral execution, mobile-first stack. */

import { useMemo, useState } from "react";
import { toast } from "sonner";
import {
  AlertCircle,
  BookOpen,
  Check,
  ChevronDown,
  Clock3,
  Code2,
  Copy,
  Download,
  ExternalLink,
  FileCode2,
  HelpCircle,
  Menu,
  Play,
  RotateCcw,
  Settings2,
  Share2,
  Sparkles,
  Terminal,
  X,
} from "lucide-react";
import { type PadmaRunResult } from "@/lib/padmaRunner";
import { runPadmaInBrowser } from "@/lib/padmaWasm";

const starter = `# Padma-তে স্বাগতম
# বাংলা ও English keyword একসঙ্গে ব্যবহার করা যায়
ধরি নাম = "রাফি"
ধরি নম্বর = ৭০ + ২৩

যদি নম্বর >= 90 {
    দেখাও "{নাম}, তুমি পেয়েছ: {নম্বর}"
} নইলে {
    দেখাও "{নাম}, আবার চেষ্টা করো।"
}`;

const englishStarter = `# Padma welcomes every developer
let name = "Rafi"
let score = 70 + 23

if score >= 90 {
    print "{name}, your score is: {score}"
} else {
    print "{name}, try again."
}`;

const mixedStarter = `# Mixed syntax is intentional
ধরি price = 250
if price > 200 {
    দেখাও "ছাড় প্রযোজ্য: {price} টাকা"
}`;

type ExampleKey = "bn" | "en" | "mixed";

const examples: Record<ExampleKey, { label: string; description: string; code: string }> = {
  bn: { label: "বাংলা শুরু", description: "একজন নতুন শিক্ষার্থীর প্রথম program", code: starter },
  en: { label: "English start", description: "A familiar syntax for global teams", code: englishStarter },
  mixed: { label: "মিশ্র syntax", description: "বাংলা ও English একই file-এ", code: mixedStarter },
};

function LineNumbers({ count }: { count: number }) {
  return <div className="line-numbers" aria-hidden="true">{Array.from({ length: Math.max(count, 1) }, (_, index) => <span key={index}>{index + 1}</span>)}</div>;
}

function AppLogo() {
  return <div className="brand-mark"><img src="/manus-storage/padma-mark_b69bb6bb.png" alt="" /><span>padma<span className="brand-dot">.</span>play</span></div>;
}

export default function Home() {
  const [code, setCode] = useState(starter);
  const [result, setResult] = useState<PadmaRunResult | null>(null);
  const [activeExample, setActiveExample] = useState<ExampleKey | null>("bn");
  const [isRunning, setIsRunning] = useState(false);
  const [showMenu, setShowMenu] = useState(false);
  const [showDocs, setShowDocs] = useState(false);
  const lines = useMemo(() => code.split("\n").length, [code]);

  const execute = () => {
    setIsRunning(true);
    window.setTimeout(() => {
      runPadmaInBrowser(code).then((nextResult) => {
        setResult(nextResult);
        setIsRunning(false);
      });
    }, 180);
  };

  const loadExample = (key: ExampleKey) => {
    setCode(examples[key].code);
    setActiveExample(key);
    setResult(null);
  };

  const copyCode = async () => {
    await navigator.clipboard.writeText(code);
    toast.success("Code copied", { description: "আপনার Padma code clipboard-এ রাখা হয়েছে।" });
  };

  const downloadCode = () => {
    const blob = new Blob([code], { type: "text/plain;charset=utf-8" });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement("a");
    anchor.href = url;
    anchor.download = "main.pd";
    anchor.click();
    URL.revokeObjectURL(url);
    toast.success("File downloaded", { description: "main.pd আপনার device-এ সংরক্ষণ করা হয়েছে।" });
  };

  const reset = () => {
    setCode(starter);
    setResult(null);
    setActiveExample("bn");
  };

  return (
    <main className="playground-shell">
      <header className="topbar">
        <div className="topbar-left"><AppLogo /><div className="version-pill"><span className="status-dot" /> v0.1 MVP</div></div>
        <div className="topbar-actions">
          <button className="icon-button mobile-menu" onClick={() => setShowMenu((value) => !value)} aria-label="Open menu">{showMenu ? <X size={17} /> : <Menu size={17} />}</button>
          <button className="plain-action hide-mobile" onClick={() => setShowDocs(true)}><BookOpen size={15} /> Docs</button>
          <button className="plain-action hide-mobile" onClick={() => toast.info("Share links are coming next", { description: "এখন code copy বা download করে share করতে পারেন।" })}><Share2 size={15} /> Share</button>
          <button className="icon-button" onClick={() => setShowDocs(true)} aria-label="Help"><HelpCircle size={17} /></button>
        </div>
      </header>

      {showMenu && <div className="mobile-menu-panel"><button onClick={() => setShowDocs(true)}><BookOpen size={16} /> Documentation</button><button onClick={copyCode}><Copy size={16} /> Copy code</button><button onClick={downloadCode}><Download size={16} /> Download .pd</button></div>}

      <section className="commandbar">
        <div className="commandbar-left">
          <button className="run-button" onClick={execute} disabled={isRunning}><Play size={15} fill="currentColor" /> {isRunning ? "RUNNING" : "RUN"}<span className="shortcut">⌘ ↵</span></button>
          <button className="select-button" onClick={() => toast.info("Padma interpreter", { description: "WASM target is being wired into this playground build." })}>Padma <ChevronDown size={14} /></button>
          <button className="select-button subtle-select" onClick={() => toast.info("Mode: debug", { description: "Debug output is enabled for the MVP." })}>DEBUG <ChevronDown size={14} /></button>
        </div>
        <div className="commandbar-right">
          <span className="runtime-chip"><span className="runtime-pulse" /> browser runtime</span>
          <button className="icon-button" onClick={reset} aria-label="Reset code"><RotateCcw size={16} /></button>
          <button className="icon-button" onClick={copyCode} aria-label="Copy code"><Copy size={16} /></button>
          <button className="icon-button" onClick={downloadCode} aria-label="Download code"><Download size={16} /></button>
          <button className="icon-button" onClick={() => setShowDocs(true)} aria-label="Settings"><Settings2 size={16} /></button>
        </div>
      </section>

      <div className="workspace">
        <aside className="sidebar">
          <div className="sidebar-heading"><span>EXAMPLES</span><Sparkles size={14} /></div>
          <div className="example-list">
            {(Object.keys(examples) as ExampleKey[]).map((key) => <button key={key} className={`example-item ${activeExample === key ? "active" : ""}`} onClick={() => loadExample(key)}><span className="example-icon"><Code2 size={14} /></span><span><strong>{examples[key].label}</strong><small>{examples[key].description}</small></span></button>)}
          </div>
          <div className="sidebar-footnote"><img src="/manus-storage/padma-learning-card_8372f890.png" alt="A learner using Padma Playground" /><div><span className="eyebrow">LEARN BY BUILDING</span><p>ভুল হলে ভয় নেই। Padma next step দেখাবে।</p></div></div>
        </aside>

        <section className="editor-column">
          <div className="file-tab"><FileCode2 size={14} /><span>main.pd</span><span className="file-state">●</span><span className="file-spacer" /><span className="editor-language">PADMA</span></div>
          <div className="editor-card">
            <LineNumbers count={lines} />
            <textarea value={code} onChange={(event) => { setCode(event.target.value); setActiveExample(null); }} spellCheck={false} aria-label="Padma code editor" />
          </div>
          <div className="editor-footer"><span><Terminal size={13} /> UTF-8</span><span>{lines} lines</span><span>Spaces: 4</span></div>
        </section>

        <section className={`output-column ${result ? "has-result" : ""}`}>
          <div className="output-header"><div><span className="eyebrow">OUTPUT</span><h2>{result?.ok ? "Program finished" : result ? "Needs attention" : "Your result appears here"}</h2></div><div className={`result-status ${result?.ok ? "success" : result ? "error" : "idle"}`}>{result?.ok ? <Check size={14} /> : result ? <AlertCircle size={14} /> : <Clock3 size={14} />}{result?.ok ? result.duration : result ? "diagnostic" : "ready"}</div></div>
          <div className="output-body">
            {!result && <><img className="run-art" src="/manus-storage/padma-run-state_32d86bf8.png" alt="" /><p className="output-empty-title">Run your Padma code</p><p className="output-empty-copy">Press <kbd>⌘</kbd> <kbd>↵</kbd> or tap Run to see output here.</p></>}
            {result?.ok && <div className="terminal-output">{result.output.length ? result.output.map((line, index) => <div className="output-line" key={`${line}-${index}`}><span className="output-caret">›</span>{line}</div>) : <div className="muted-output">Program completed without printed output.</div>}</div>}
            {result && !result.ok && <div className="diagnostic-box"><div className="diagnostic-label"><AlertCircle size={14} /> {result.locale === "bn" ? "Padma diagnostic" : "Padma diagnostic"}</div>{result.diagnostics.map((line) => <pre key={line}>{line}</pre>)}<button onClick={() => setShowDocs(true)}>Read the language guide <ExternalLink size={13} /></button></div>}
          </div>
          <div className="output-footer"><span><span className="tiny-dot" /> {result ? (result.ok ? "Execution complete" : "Execution stopped") : "Waiting for a run"}</span><span>WASM-ready runtime</span></div>
        </section>
      </div>

      <footer className="bottom-strip"><span><span className="footer-logo">◆</span> Padma is an open language for Bangladesh and beyond.</span><span className="footer-links"><a href="https://github.com/OfficialBiohub/padma-lang" target="_blank" rel="noreferrer">GitHub <ExternalLink size={12} /></a><button onClick={() => setShowDocs(true)}>How it works <ExternalLink size={12} /></button></span></footer>

      {showDocs && <div className="modal-backdrop" onClick={() => setShowDocs(false)}><div className="docs-modal" onClick={(event) => event.stopPropagation()}><div className="modal-top"><div><span className="eyebrow">PADMA PLAYGROUND</span><h2>Write once. Understand more.</h2></div><button className="icon-button" onClick={() => setShowDocs(false)} aria-label="Close documentation"><X size={17} /></button></div><p>Padma accepts বাংলা, English, or a thoughtful mix of both. The browser runner currently supports variables, arithmetic, conditions, strings, interpolation, Bengali digits, and bilingual diagnostics.</p><div className="docs-grid"><div><strong>Run</strong><span>⌘ ↵</span></div><div><strong>Copy</strong><span>⌘ C</span></div><div><strong>Reset</strong><span>⌘ R</span></div></div><a className="docs-link" href="https://github.com/OfficialBiohub/padma-lang" target="_blank" rel="noreferrer">Open the compiler repository <ExternalLink size={14} /></a></div></div>}
    </main>
  );
}
