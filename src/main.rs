mod gacha;
mod types;

use types::{Pool, PoolState};

fn main() {
    // 30 items: 2 jackpot + 5 gold + 5 silver + 18 normal.
    let pool = Pool {
        num_items: 30,
        jackpot_items: 2,
        gold_items: 5,
        silver_items: 5,
        normal_items: 18,
        soft_pity: 20,
        hard_pity: 25,
        multiplier: 1.15,
    };
    let mut state = PoolState::new(&pool);

    println!(
        "Pool: {} jackpot, {} gold, {} silver, {} normal ({} total)",
        pool.jackpot_items, pool.gold_items, pool.silver_items, pool.normal_items, pool.num_items
    );
    println!(
        "Jackpot chance on pull 1: {:.2}%",
        pool.pull_probability(&state) * 100.0
    );
    println!(
        "At least one jackpot within 10 pulls: {:.2}%",
        pool.jackpot_within(&state, 10) * 100.0
    );
    println!(
        "Average pulls until the next jackpot: {:.2}",
        pool.average_jackpot(&state)
    );

    // This records a sample non-jackpot result. A random draw can be added later.
    state.record_pull(false);
    println!(
        "After one miss, pull 2 jackpot chance: {:.2}%",
        pool.pull_probability(&state) * 100.0
    );
}
