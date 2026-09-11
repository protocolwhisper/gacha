use crate::types::{Pool, PoolState};

impl Pool {
    pub fn pull_probability(&self, state: &PoolState) -> f32 {
        if state.remaining_items == 0 || state.remaining_jackpots == 0 {
            return 0.0;
        }

        let pull = state.misses_since_jackpot + 1;

        if pull >= self.hard_pity || state.remaining_items == state.remaining_jackpots {
            1.0
        } else if pull <= self.soft_pity {
            self.normal_distribution(state)
        } else {
            self.soft_pity(state, pull)
        }
    }

    fn normal_distribution(&self, state: &PoolState) -> f32 {
        state.remaining_jackpots as f32 / state.remaining_items as f32
    }

    fn soft_pity(&self, state: &PoolState, pull: u32) -> f32 {
        let jackpot_weight = self.multiplier.powf((pull - self.soft_pity) as f32);
        let total_jackpot_weight = state.remaining_jackpots as f32 * jackpot_weight;
        let non_jackpot_weight = (state.remaining_items - state.remaining_jackpots) as f32;

        total_jackpot_weight / (non_jackpot_weight + total_jackpot_weight)
    }

    // Probability of at least one jackpot within the next `pulls` pulls.
    pub fn consecutive_probability(&self, state: &PoolState, pulls: u32) -> f32 {
        let mut missed_every_pull = 1.0;
        let mut simulated_state = state.clone();

        for _ in 0..pulls {
            let probability = self.pull_probability(&simulated_state);
            missed_every_pull *= 1.0 - probability;

            if probability == 1.0 {
                break;
            }

            if !simulated_state.record_pull(false) {
                break;
            }
        }

        1.0 - missed_every_pull
    }

    // More explicit alias for `consecutive_probability`.
    pub fn jackpot_within(&self, state: &PoolState, pulls: u32) -> f32 {
        self.consecutive_probability(state, pulls)
    }

    // Probability that the next jackpot occurs exactly on future pull `pull`.
    pub fn at_pull(&self, state: &PoolState, pull: u32) -> f32 {
        if pull == 0 {
            return 0.0;
        }

        let mut missed_previous = 1.0;
        let mut simulated_state = state.clone();

        for current_pull in 1..=pull {
            let probability = self.pull_probability(&simulated_state);

            if current_pull == pull {
                return missed_previous * probability;
            }

            missed_previous *= 1.0 - probability;
            if probability == 1.0 || !simulated_state.record_pull(false) {
                return 0.0;
            }
        }

        0.0
    }

    pub fn average_jackpot(&self, state: &PoolState) -> f32 {
        (1..=state.remaining_items)
            .map(|pull| pull as f32 * self.at_pull(state, pull))
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example_pool() -> Pool {
        Pool {
            num_items: 30,
            jackpot_items: 2,
            gold_items: 5,
            silver_items: 5,
            normal_items: 18,
            soft_pity: 20,
            hard_pity: 25,
            multiplier: 1.15,
        }
    }

    #[test]
    fn initial_probability_includes_both_jackpots() {
        let pool = example_pool();
        let state = PoolState::new(&pool);

        assert!((pool.pull_probability(&state) - 2.0 / 30.0).abs() < 0.000_001);
    }

    #[test]
    fn consecutive_probability_multiplies_misses() {
        let pool = example_pool();
        let state = PoolState::new(&pool);
        let expected = 1.0 - (28.0 / 30.0) * (27.0 / 29.0);

        assert!((pool.jackpot_within(&state, 2) - expected).abs() < 0.000_001);
    }

    #[test]
    fn exact_pull_requires_previous_misses() {
        let pool = example_pool();
        let state = PoolState::new(&pool);
        let expected = (28.0 / 30.0) * (2.0 / 29.0);

        assert!((pool.at_pull(&state, 2) - expected).abs() < 0.000_001);
    }

    #[test]
    fn soft_pity_weights_every_remaining_jackpot() {
        let pool = example_pool();
        let mut state = PoolState::new(&pool);

        for _ in 0..20 {
            assert!(state.record_pull(false));
        }

        let expected = (2.0 * 1.15) / (8.0 + 2.0 * 1.15);
        assert!((pool.pull_probability(&state) - expected).abs() < 0.000_001);
    }

    #[test]
    fn jackpot_updates_state_and_resets_pity() {
        let pool = example_pool();
        let mut state = PoolState::new(&pool);

        assert!(state.record_pull(false));
        assert!(state.record_pull(true));

        assert_eq!(state.remaining_items, 28);
        assert_eq!(state.remaining_jackpots, 1);
        assert_eq!(state.pulls_made, 2);
        assert_eq!(state.misses_since_jackpot, 0);
        assert!((pool.pull_probability(&state) - 1.0 / 28.0).abs() < 0.000_001);
    }

    #[test]
    fn hard_pity_guarantees_the_next_pull() {
        let pool = example_pool();
        let mut state = PoolState::new(&pool);

        for _ in 0..24 {
            assert!(state.record_pull(false));
        }

        assert_eq!(pool.pull_probability(&state), 1.0);
    }

    #[test]
    fn exact_pull_probabilities_sum_to_one() {
        let pool = example_pool();
        let state = PoolState::new(&pool);
        let total: f32 = (1..=pool.hard_pity)
            .map(|pull| pool.at_pull(&state, pull))
            .sum();

        assert!((total - 1.0).abs() < 0.000_001);
    }
}
