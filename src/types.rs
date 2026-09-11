#[allow(dead_code)]
struct Item {
    value: u32,
    is_rare: bool,
    is_jackpot: bool,
}

#[derive(Debug)]
pub struct Pool {
    pub num_items: u32,
    pub jackpot_items: u32,
    pub gold_items: u32,
    pub silver_items: u32,
    pub normal_items: u32,
    pub soft_pity: u32,
    pub hard_pity: u32,
    pub multiplier: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoolState {
    pub remaining_items: u32,
    pub remaining_jackpots: u32,
    pub pulls_made: u32,
    pub misses_since_jackpot: u32,
}

impl PoolState {
    pub fn new(pool: &Pool) -> Self {
        Self {
            remaining_items: pool.num_items,
            remaining_jackpots: pool.jackpot_items,
            pulls_made: 0,
            misses_since_jackpot: 0,
        }
    }

    pub fn record_pull(&mut self, is_jackpot: bool) -> bool {
        if self.remaining_items == 0 {
            return false;
        }

        if is_jackpot {
            if self.remaining_jackpots == 0 {
                return false;
            }
            self.remaining_jackpots -= 1;
            self.misses_since_jackpot = 0;
        } else {
            if self.remaining_items == self.remaining_jackpots {
                return false;
            }
            self.misses_since_jackpot += 1;
        }

        self.remaining_items -= 1;
        self.pulls_made += 1;
        true
    }
}
