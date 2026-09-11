struct Item {
    value: u32,
    is_rare: bool,
    is_jackpot: bool
}

pub struct Pool{
    pub num_items: u32,
    pub max_price: u32,
    pub soft_pity: u32,
    pub hard_pity: u32,
    pub multiplier: f32,
    pub floor_price:u32
}



