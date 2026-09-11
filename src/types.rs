#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Item {
    Jackpot,
    Gold,
    Silver,
    Normal,
}

#[derive(Debug, Clone)]
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

impl Pool {
    pub fn validate(&self) -> Result<(), &'static str> {
        let categorized_items = self
            .jackpot_items
            .checked_add(self.gold_items)
            .and_then(|count| count.checked_add(self.silver_items))
            .and_then(|count| count.checked_add(self.normal_items))
            .ok_or("item counts are too large")?;

        if self.num_items == 0 || categorized_items != self.num_items {
            return Err("tier counts must add up to the total item count");
        }
        if self.jackpot_items == 0 {
            return Err("the pool needs at least one jackpot");
        }
        if self.soft_pity >= self.hard_pity {
            return Err("soft pity must start before hard pity");
        }
        if self.hard_pity == 0 || self.hard_pity > self.num_items - self.jackpot_items + 1 {
            return Err("hard pity must be reachable before natural pool exhaustion");
        }
        if !self.multiplier.is_finite() || self.multiplier < 1.0 {
            return Err("the soft-pity multiplier must be at least 1");
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoolState {
    pub remaining_items: u32,
    pub remaining_jackpots: u32,
    pub remaining_gold: u32,
    pub remaining_silver: u32,
    pub remaining_normal: u32,
    pub pulls_made: u32,
    pub misses_since_jackpot: u32,
}

impl PoolState {
    pub fn new(pool: &Pool) -> Self {
        Self {
            remaining_items: pool.num_items,
            remaining_jackpots: pool.jackpot_items,
            remaining_gold: pool.gold_items,
            remaining_silver: pool.silver_items,
            remaining_normal: pool.normal_items,
            pulls_made: 0,
            misses_since_jackpot: 0,
        }
    }

    pub fn record_pull(&mut self, item: Item) -> bool {
        if self.remaining_items == 0 {
            return false;
        }

        let remaining_tier = match item {
            Item::Jackpot => &mut self.remaining_jackpots,
            Item::Gold => &mut self.remaining_gold,
            Item::Silver => &mut self.remaining_silver,
            Item::Normal => &mut self.remaining_normal,
        };

        if *remaining_tier == 0 {
            return false;
        }
        *remaining_tier -= 1;

        if item == Item::Jackpot {
            self.misses_since_jackpot = 0;
        } else {
            self.misses_since_jackpot += 1;
        }

        self.remaining_items -= 1;
        self.pulls_made += 1;
        true
    }

    pub(crate) fn record_miss(&mut self) -> bool {
        let item = if self.remaining_normal > 0 {
            Item::Normal
        } else if self.remaining_silver > 0 {
            Item::Silver
        } else if self.remaining_gold > 0 {
            Item::Gold
        } else {
            return false;
        };

        self.record_pull(item)
    }
}
