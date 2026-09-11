use gacha::{Item, Pool, PoolState};
use rand::{SeedableRng, rngs::StdRng};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Debug, Deserialize)]
struct Config {
    jackpot_items: u32,
    gold_items: u32,
    silver_items: u32,
    normal_items: u32,
    soft_pity: u32,
    hard_pity: u32,
    multiplier: f32,
}

#[derive(Serialize)]
struct StateSnapshot {
    remaining_items: u32,
    remaining_jackpots: u32,
    remaining_gold: u32,
    remaining_silver: u32,
    remaining_normal: u32,
    pulls_made: u32,
    misses_since_jackpot: u32,
    current_jackpot_probability: f32,
    average_pulls_to_jackpot: f32,
    seed: String,
}

#[derive(Serialize)]
struct PullResult {
    item: &'static str,
    pull_number: u32,
    jackpot_probability: f32,
    state: StateSnapshot,
}

#[derive(Serialize)]
struct AnalyticsPoint {
    pull: u32,
    conditional: f32,
    cumulative: f32,
    exact: f32,
}

#[wasm_bindgen]
pub struct GachaEngine {
    pool: Pool,
    state: PoolState,
    rng: StdRng,
    seed: u64,
}

#[wasm_bindgen]
impl GachaEngine {
    #[wasm_bindgen(constructor)]
    pub fn new(config: JsValue, seed: String) -> Result<GachaEngine, JsValue> {
        let config: Config = serde_wasm_bindgen::from_value(config)
            .map_err(|error| js_error(&format!("invalid configuration: {error}")))?;
        let seed = seed
            .parse::<u64>()
            .map_err(|_| js_error("seed must be an unsigned 64-bit integer"))?;
        let pool = pool_from_config(config)?;

        Ok(Self {
            state: PoolState::new(&pool),
            pool,
            rng: StdRng::seed_from_u64(seed),
            seed,
        })
    }

    pub fn pull(&mut self) -> Result<JsValue, JsValue> {
        let probability = self.pool.pull_probability(&self.state);
        let item = self
            .pool
            .pull(&mut self.state, &mut self.rng)
            .ok_or_else(|| js_error("the pool is empty"))?;
        let result = PullResult {
            item: item_name(item),
            pull_number: self.state.pulls_made,
            jackpot_probability: probability,
            state: self.snapshot_value(),
        };

        serde_wasm_bindgen::to_value(&result).map_err(|error| js_error(&error.to_string()))
    }

    pub fn pull_many(&mut self, count: u32) -> Result<JsValue, JsValue> {
        let mut results = Vec::new();
        for _ in 0..count {
            if self.state.remaining_items == 0 {
                break;
            }

            let probability = self.pool.pull_probability(&self.state);
            let item = self
                .pool
                .pull(&mut self.state, &mut self.rng)
                .ok_or_else(|| js_error("the pool is empty"))?;
            results.push(PullResult {
                item: item_name(item),
                pull_number: self.state.pulls_made,
                jackpot_probability: probability,
                state: self.snapshot_value(),
            });
        }

        serde_wasm_bindgen::to_value(&results).map_err(|error| js_error(&error.to_string()))
    }

    pub fn snapshot(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.snapshot_value())
            .map_err(|error| js_error(&error.to_string()))
    }

    pub fn analytics(&self) -> Result<JsValue, JsValue> {
        let max_before_natural_guarantee = self
            .state
            .remaining_items
            .saturating_sub(self.state.remaining_jackpots)
            + 1;
        let until_hard_pity = self
            .pool
            .hard_pity
            .saturating_sub(self.state.misses_since_jackpot)
            .max(1);
        let horizon = max_before_natural_guarantee.min(until_hard_pity);
        let mut previous_cumulative = 0.0;
        let mut points = Vec::with_capacity(horizon as usize);

        for pull in 1..=horizon {
            let cumulative = self.pool.jackpot_within(&self.state, pull);
            let exact = self.pool.at_pull(&self.state, pull);
            let survival = 1.0 - previous_cumulative;
            let conditional = if survival > 0.0 {
                (exact / survival).clamp(0.0, 1.0)
            } else {
                0.0
            };
            points.push(AnalyticsPoint {
                pull,
                conditional,
                cumulative,
                exact,
            });
            previous_cumulative = cumulative;
        }

        serde_wasm_bindgen::to_value(&points).map_err(|error| js_error(&error.to_string()))
    }
}

impl GachaEngine {
    fn snapshot_value(&self) -> StateSnapshot {
        StateSnapshot {
            remaining_items: self.state.remaining_items,
            remaining_jackpots: self.state.remaining_jackpots,
            remaining_gold: self.state.remaining_gold,
            remaining_silver: self.state.remaining_silver,
            remaining_normal: self.state.remaining_normal,
            pulls_made: self.state.pulls_made,
            misses_since_jackpot: self.state.misses_since_jackpot,
            current_jackpot_probability: self.pool.pull_probability(&self.state),
            average_pulls_to_jackpot: self.pool.average_jackpot(&self.state),
            seed: self.seed.to_string(),
        }
    }
}

fn pool_from_config(config: Config) -> Result<Pool, JsValue> {
    let total_items = config
        .jackpot_items
        .checked_add(config.gold_items)
        .and_then(|count| count.checked_add(config.silver_items))
        .and_then(|count| count.checked_add(config.normal_items))
        .ok_or_else(|| js_error("item counts are too large"))?;
    let pool = Pool {
        num_items: total_items,
        jackpot_items: config.jackpot_items,
        gold_items: config.gold_items,
        silver_items: config.silver_items,
        normal_items: config.normal_items,
        soft_pity: config.soft_pity,
        hard_pity: config.hard_pity,
        multiplier: config.multiplier,
    };
    pool.validate().map_err(js_error)?;
    Ok(pool)
}

fn item_name(item: Item) -> &'static str {
    match item {
        Item::Jackpot => "Jackpot",
        Item::Gold => "Gold",
        Item::Silver => "Silver",
        Item::Normal => "Normal",
    }
}

fn js_error(message: &str) -> JsValue {
    js_sys::Error::new(message).into()
}
