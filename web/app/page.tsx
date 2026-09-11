"use client";

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import Chart from "chart.js/auto";
import katex from "katex";

type Config = {
  jackpot_items: number;
  gold_items: number;
  silver_items: number;
  normal_items: number;
  soft_pity: number;
  hard_pity: number;
  multiplier: number;
};

type Snapshot = {
  remaining_items: number;
  remaining_jackpots: number;
  remaining_gold: number;
  remaining_silver: number;
  remaining_normal: number;
  pulls_made: number;
  misses_since_jackpot: number;
  current_jackpot_probability: number;
  average_pulls_to_jackpot: number;
  seed: string;
};

type PullResult = {
  item: "Jackpot" | "Gold" | "Silver" | "Normal";
  pull_number: number;
  jackpot_probability: number;
  state: Snapshot;
};

type AnalyticsPoint = {
  pull: number;
  conditional: number;
  cumulative: number;
  exact: number;
};

type Engine = {
  pull(): PullResult;
  pull_many(count: number): PullResult[];
  snapshot(): Snapshot;
  analytics(): AnalyticsPoint[];
};

type WasmModule = {
  default(): Promise<unknown>;
  GachaEngine: new (config: Config, seed: string) => Engine;
};

const DEFAULT_CONFIG: Config = {
  jackpot_items: 2,
  gold_items: 5,
  silver_items: 5,
  normal_items: 18,
  soft_pity: 20,
  hard_pity: 25,
  multiplier: 1.15,
};

const TIERS = ["Jackpot", "Gold", "Silver", "Normal"] as const;

function freshSeed() {
  const words = new Uint32Array(2);
  crypto.getRandomValues(words);
  return ((BigInt(words[0]) << 32n) | BigInt(words[1])).toString();
}

function percent(value: number, digits = 2) {
  return `${(value * 100).toFixed(digits)}%`;
}

function Formula({ expression }: { expression: string }) {
  const html = useMemo(
    () => katex.renderToString(expression, { throwOnError: false }),
    [expression],
  );
  return <span className="formula" dangerouslySetInnerHTML={{ __html: html }} />;
}

function ProbabilityChart({
  title,
  description,
  points,
  field,
  color,
}: {
  title: string;
  description: string;
  points: AnalyticsPoint[];
  field: "conditional" | "cumulative" | "exact";
  color: string;
}) {
  const canvas = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    if (!canvas.current || points.length === 0) return;
    const chart = new Chart(canvas.current, {
      type: "line",
      data: {
        labels: points.map((point) => point.pull),
        datasets: [{
          data: points.map((point) => point[field] * 100),
          borderColor: color,
          backgroundColor: `${color}22`,
          fill: true,
          borderWidth: 2,
          pointRadius: points.length > 30 ? 0 : 2,
          pointHoverRadius: 5,
          tension: 0.28,
        }],
      },
      options: {
        animation: { duration: 450 },
        responsive: true,
        maintainAspectRatio: false,
        plugins: {
          legend: { display: false },
          tooltip: { callbacks: { label: (context) => `${Number(context.raw).toFixed(2)}%` } },
        },
        scales: {
          x: {
            title: { display: true, text: "Future pull", color: "#7c879d" },
            grid: { color: "rgba(255,255,255,.045)" },
            ticks: { color: "#7c879d", maxTicksLimit: 8 },
          },
          y: {
            beginAtZero: true,
            max: field === "cumulative" ? 100 : undefined,
            grid: { color: "rgba(255,255,255,.06)" },
            ticks: { color: "#7c879d", callback: (value) => `${value}%` },
          },
        },
      },
    });
    return () => chart.destroy();
  }, [color, field, points]);

  return (
    <article className="chart-card">
      <div className="section-heading compact">
        <div><span className="eyebrow">{field}</span><h3>{title}</h3></div>
        <p>{description}</p>
      </div>
      <div className="chart-wrap"><canvas ref={canvas} /></div>
    </article>
  );
}

function parseUrlConfig(): { config: Config; seed?: string } {
  const params = new URLSearchParams(window.location.search);
  const number = (key: keyof Config, fallback: number) => {
    const raw = params.get(key);
    if (raw === null) return fallback;
    const parsed = Number(raw);
    return Number.isFinite(parsed) ? parsed : fallback;
  };
  return {
    config: {
      jackpot_items: number("jackpot_items", DEFAULT_CONFIG.jackpot_items),
      gold_items: number("gold_items", DEFAULT_CONFIG.gold_items),
      silver_items: number("silver_items", DEFAULT_CONFIG.silver_items),
      normal_items: number("normal_items", DEFAULT_CONFIG.normal_items),
      soft_pity: number("soft_pity", DEFAULT_CONFIG.soft_pity),
      hard_pity: number("hard_pity", DEFAULT_CONFIG.hard_pity),
      multiplier: number("multiplier", DEFAULT_CONFIG.multiplier),
    },
    seed: params.get("seed") ?? undefined,
  };
}

export default function Home() {
  const [config, setConfig] = useState(DEFAULT_CONFIG);
  const [seed, setSeed] = useState("loading");
  const [snapshot, setSnapshot] = useState<Snapshot | null>(null);
  const [analytics, setAnalytics] = useState<AnalyticsPoint[]>([]);
  const [history, setHistory] = useState<PullResult[]>([]);
  const [error, setError] = useState("");
  const [copyLabel, setCopyLabel] = useState("Copy setup link");
  const wasm = useRef<WasmModule | null>(null);
  const engine = useRef<Engine | null>(null);

  const totalItems = config.jackpot_items + config.gold_items + config.silver_items + config.normal_items;

  const syncEngine = useCallback(() => {
    if (!engine.current) return;
    setSnapshot(engine.current.snapshot());
    setAnalytics(engine.current.analytics());
  }, []);

  const createEngine = useCallback((nextConfig: Config, nextSeed: string) => {
    if (!wasm.current) return;
    try {
      engine.current = new wasm.current.GachaEngine(nextConfig, nextSeed);
      setConfig(nextConfig);
      setSeed(nextSeed);
      setHistory([]);
      setError("");
      setSnapshot(engine.current.snapshot());
      setAnalytics(engine.current.analytics());
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    }
  }, []);

  useEffect(() => {
    let cancelled = false;
    const start = async () => {
      try {
        const wasmBindings = (await import("../wasm/gacha_web.js")) as WasmModule;
        await wasmBindings.default();
        if (cancelled) return;
        wasm.current = wasmBindings;
        const url = parseUrlConfig();
        createEngine(url.config, url.seed ?? freshSeed());
      } catch (cause) {
        setError(`Could not load the Rust engine: ${String(cause)}`);
      }
    };
    void start();
    return () => { cancelled = true; };
  }, [createEngine]);

  const updateConfig = (key: keyof Config, value: string) => {
    setConfig((current) => ({ ...current, [key]: Number(value) }));
  };

  const reset = (newRandomSeed = false) => {
    createEngine(config, newRandomSeed ? freshSeed() : seed);
  };

  const draw = (count: number) => {
    if (!engine.current) return;
    try {
      const results = count === 1 ? [engine.current.pull()] : engine.current.pull_many(count);
      setHistory((current) => [...current, ...results]);
      setError("");
      syncEngine();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    }
  };

  const copySetup = async () => {
    const params = new URLSearchParams();
    Object.entries(config).forEach(([key, value]) => params.set(key, String(value)));
    params.set("seed", seed);
    await navigator.clipboard.writeText(`${window.location.origin}${window.location.pathname}?${params}`);
    setCopyLabel("Copied");
    window.setTimeout(() => setCopyLabel("Copy setup link"), 1500);
  };

  const inventory = snapshot ? [
    ["Jackpot", snapshot.remaining_jackpots],
    ["Gold", snapshot.remaining_gold],
    ["Silver", snapshot.remaining_silver],
    ["Normal", snapshot.remaining_normal],
  ] as const : [];

  return (
    <main>
      <header className="hero shell">
        <nav>
          <div className="brand"><span className="brand-mark">G</span> Gacha Lab</div>
          <span className="tech-pill">Rust → WebAssembly</span>
        </nav>
        <div className="hero-grid">
          <div>
            <span className="eyebrow">Probability you can play</span>
            <h1>See every pull.<br /><em>Understand every chance.</em></h1>
            <p className="hero-copy">A live, without-replacement gacha model with multiple jackpots, exponential soft pity, and a hard guarantee.</p>
          </div>
          <div className="hero-stat">
            <span>Current jackpot chance</span>
            <strong>{snapshot ? percent(snapshot.current_jackpot_probability) : "—"}</strong>
            <small>on the next pull</small>
          </div>
        </div>
      </header>

      <section className="shell workspace">
        <aside className="control-panel panel">
          <div className="section-heading">
            <div><span className="eyebrow">01 / Configure</span><h2>Build the pool</h2></div>
            <span className="total-chip">{totalItems} items</span>
          </div>
          <div className="input-grid">
            {TIERS.map((tier) => {
              const key = `${tier.toLowerCase()}_items` as keyof Config;
              return (
                <label key={tier}>
                  <span><i className={`tier-dot ${tier.toLowerCase()}`} />{tier}</span>
                  <input aria-label={`${tier} items`} type="number" min={tier === "Jackpot" ? 1 : 0} step="1" value={config[key]} onChange={(event) => updateConfig(key, event.target.value)} />
                </label>
              );
            })}
            <label><span>Soft pity misses</span><input aria-label="Soft pity misses" type="number" min="0" step="1" value={config.soft_pity} onChange={(event) => updateConfig("soft_pity", event.target.value)} /></label>
            <label><span>Hard pity pull</span><input aria-label="Hard pity pull" type="number" min="1" step="1" value={config.hard_pity} onChange={(event) => updateConfig("hard_pity", event.target.value)} /></label>
            <label><span>Weight multiplier</span><input aria-label="Weight multiplier" type="number" min="1" step="0.01" value={config.multiplier} onChange={(event) => updateConfig("multiplier", event.target.value)} /></label>
            <label><span>Random seed</span><input aria-label="Random seed" type="text" inputMode="numeric" value={seed} onChange={(event) => setSeed(event.target.value)} /></label>
          </div>
          {error && <p className="error" role="alert">{error}</p>}
          <div className="control-actions">
            <button className="button primary" onClick={() => reset(false)} disabled={!snapshot}>Apply & reset</button>
            <button className="button ghost" onClick={() => reset(true)} disabled={!snapshot}>New seed</button>
          </div>
          <button className="text-button" onClick={copySetup} disabled={!snapshot}>{copyLabel} ↗</button>
        </aside>

        <div className="play-column">
          <section className="panel draw-panel">
            <div className="section-heading">
              <div><span className="eyebrow">02 / Draw</span><h2>Run the machine</h2></div>
              <span className="pull-counter">{snapshot?.pulls_made ?? 0} pulls</span>
            </div>
            <div className="metrics">
              <div><span>Next pull</span><strong>{snapshot ? percent(snapshot.current_jackpot_probability) : "—"}</strong></div>
              <div><span>Average wait</span><strong>{snapshot ? snapshot.average_pulls_to_jackpot.toFixed(2) : "—"}</strong></div>
              <div><span>Pity misses</span><strong>{snapshot?.misses_since_jackpot ?? "—"}</strong></div>
              <div><span>Remaining</span><strong>{snapshot?.remaining_items ?? "—"}</strong></div>
            </div>
            <div className="inventory" aria-label="Remaining inventory">
              {inventory.map(([tier, count]) => (
                <div className={`inventory-row ${tier.toLowerCase()}`} key={tier}>
                  <span>{tier}</span><div><i style={{ width: `${snapshot ? (count / Math.max(snapshot.remaining_items, 1)) * 100 : 0}%` }} /></div><b>{count}</b>
                </div>
              ))}
            </div>
            <div className="draw-actions">
              <button className="button jackpot-button" onClick={() => draw(1)} disabled={!snapshot || snapshot.remaining_items === 0}>Pull once</button>
              <button className="button secondary" onClick={() => draw(10)} disabled={!snapshot || snapshot.remaining_items === 0}>Pull ×10</button>
            </div>
          </section>

          <section className="panel history-panel">
            <div className="history-header"><span>Pull history</span><small>probability before each draw</small></div>
            {history.length === 0 ? <div className="empty-history">Your rewards will appear here.</div> : (
              <div className="history-list">
                {[...history].reverse().map((result) => (
                  <div className={`history-item ${result.item.toLowerCase()}`} key={result.pull_number}>
                    <span className="history-number">#{result.pull_number}</span><strong>{result.item}</strong><span>{percent(result.jackpot_probability)}</span>
                  </div>
                ))}
              </div>
            )}
          </section>
        </div>
      </section>

      <section className="shell analytics-section">
        <div className="section-heading wide">
          <div><span className="eyebrow">03 / Inspect</span><h2>The probability, from three angles</h2></div>
          <p>Each curve recalculates from the live pool state after every draw.</p>
        </div>
        <div className="charts-grid">
          <ProbabilityChart title="Chance on this pull" description="If every earlier pull missed." points={analytics} field="conditional" color="#75e6b8" />
          <ProbabilityChart title="Won by this pull" description="At least one jackpot so far." points={analytics} field="cumulative" color="#f4cb69" />
          <ProbabilityChart title="Exactly on this pull" description="All earlier pulls miss, then this wins." points={analytics} field="exact" color="#c49bff" />
        </div>
      </section>

      <section className="shell formulas-section">
        <div className="section-heading wide">
          <div><span className="eyebrow">04 / Understand</span><h2>The formulas behind the curves</h2></div>
          <p>The browser is not copying the Rust logic. It calls the compiled Rust engine directly.</p>
        </div>
        <div className="formula-grid">
          <article><span className="formula-index">A</span><h3>Next-pull chance</h3><Formula expression={String.raw`P_t=\frac{K w_t}{R+K w_t}`} /><p>The combined weight of the remaining jackpots divided by all remaining weight.</p></article>
          <article><span className="formula-index">B</span><h3>Soft-pity weight</h3><Formula expression={String.raw`w_t=\alpha^{\,t-S}`} /><p>After S misses, the jackpot weight grows exponentially by multiplier α.</p></article>
          <article><span className="formula-index">C</span><h3>At least one by n</h3><Formula expression={String.raw`P(T\le n)=1-\prod_{i=1}^{n}(1-P_i)`} /><p>One minus the probability of missing every pull through n.</p></article>
          <article><span className="formula-index">D</span><h3>Exactly at n</h3><Formula expression={String.raw`P(T=n)=P_n\prod_{i=1}^{n-1}(1-P_i)`} /><p>Miss every earlier pull, then win specifically on pull n.</p></article>
        </div>
      </section>

      <footer className="shell"><span>Gacha Lab</span><p>A transparent probability model built with Rust and WebAssembly.</p></footer>
    </main>
  );
}
