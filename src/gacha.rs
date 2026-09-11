use crate::types::Pool;

impl Pool{
    fn pull_probability(&self, pull: u32) -> f32{
        if pull <= self.soft_pity{
            self.normal_distribution(pull)
        } else if pull > self.soft_pity && pull < self.hard_pity  {
            self.soft_pity(pull)            
        } else {
            1.0
        }

    }

    fn normal_distribution(&self, pull: u32) -> f32{
        1.00 /(self.num_items - pull +1) as f32
    }

    fn soft_pity(&self, pull: u32) -> f32 {
        let fpull = pull as f32;
        let pity = self.multiplier.powf(fpull  - self.soft_pity as f32);
        pity / (self.num_items as f32 - pull as f32 + pity)
    }

    fn consecutive_probability(&self , pull: u32, ) -> f32{
        let mut probability = self.pull_probability(pull);
        for n in 2..pull{
            probability = probability * self.pull_probability(n) 
        }
        1.00 - probability
    }

    //Probability at n consecutive pulls
    fn consecutive_pulls(&self , pull: u32) -> f32{
        1.0 - self.consecutive_probability(pull)
    }

    //Jackpot probability at specifically n pull
    fn at_pull(&self,pull: u32) -> f32{
        let mut probability: f32 = 0.0;
        for n in 1..pull-1{
            probability = self.pull_probability(n)
        }
        probability * self.pull_probability(pull)
        
    }

    fn average_jackpot(&self) -> f32 {
        for n in 1..self.hard_pity
    }


}


