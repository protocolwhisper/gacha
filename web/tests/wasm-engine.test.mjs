import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

import init, { GachaEngine } from "../wasm/gacha_web.js";

const config = {
  jackpot_items: 2,
  gold_items: 5,
  silver_items: 5,
  normal_items: 18,
  soft_pity: 20,
  hard_pity: 25,
  multiplier: 1.15,
};

const wasm = await readFile(new URL("../wasm/gacha_web_bg.wasm", import.meta.url));
await init({ module_or_path: wasm });

test("the browser adapter exposes Rust state and analytics", () => {
  const engine = new GachaEngine(config, "7");
  const state = engine.snapshot();
  const analytics = engine.analytics();

  assert.equal(state.remaining_items, 30);
  assert.equal(state.remaining_jackpots, 2);
  assert.ok(Math.abs(state.current_jackpot_probability - 2 / 30) < 0.000001);
  assert.equal(analytics.at(-1).cumulative, 1);
  engine.free();
});

test("equal seeds replay the same Rust pull sequence", () => {
  const first = new GachaEngine(config, "99123");
  const second = new GachaEngine(config, "99123");

  const firstSequence = first.pull_many(15).map((result) => result.item);
  const secondSequence = second.pull_many(15).map((result) => result.item);

  assert.deepEqual(firstSequence, secondSequence);
  assert.equal(first.snapshot().remaining_items, 15);
  first.free();
  second.free();
});
