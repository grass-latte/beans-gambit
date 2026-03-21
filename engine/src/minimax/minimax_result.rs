use std::ops::Neg;

#[derive(Copy, Clone, Debug)]
pub struct MinimaxResult {
    pub score: f32,
    pub poisoned: bool, // Don't add to cache - impure considerations affected eval
}

impl MinimaxResult {
    pub fn new(score: f32, poisoned: bool) -> MinimaxResult {
        MinimaxResult { score, poisoned }
    }

    pub fn normal(score: f32) -> MinimaxResult {
        MinimaxResult {
            score,
            poisoned: false,
        }
    }

    pub fn poisoned(score: f32) -> MinimaxResult {
        MinimaxResult {
            score,
            poisoned: true,
        }
    }
}

impl Neg for MinimaxResult {
    type Output = MinimaxResult;
    fn neg(self) -> MinimaxResult {
        Self {
            score: -self.score,
            poisoned: self.poisoned,
        }
    }
}
