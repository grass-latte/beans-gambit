#[cfg(debug_assertions)]
use std::backtrace::Backtrace;
use std::cmp::Ordering;
use std::ops::Neg;

// TODO: Optimise into f32 / smaller type?
// TODO: Write tests
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Score {
    PositiveMateIn(u8), // +inf
    Score(f32),         // score
    NegativeMateIn(u8), // -inf
}

impl Score {
    pub fn increment_mate_in(self) -> Score {
        match self {
            Score::PositiveMateIn(pmi) => Score::PositiveMateIn(pmi + 1),
            Score::Score(s) => Score::Score(s),
            Score::NegativeMateIn(nmi) => Score::NegativeMateIn(nmi + 1),
        }
    }

    pub const ZERO: Score = Score::Score(0.0);
    pub const BADE_MATE_IN_ZERO: Score = Score::NegativeMateIn(0);
    pub const NEG_INF: Score = Score::NegativeMateIn(0);
}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self {
            Score::PositiveMateIn(pmi) => match other {
                Score::PositiveMateIn(opmi) => Some(opmi.cmp(pmi)), // We want a lower good mate
                _ => Some(Ordering::Greater),
            },
            Score::Score(s) => match other {
                Score::PositiveMateIn(_) => Some(Ordering::Less),
                Score::Score(os) => s.partial_cmp(os), // We want a bigger score
                Score::NegativeMateIn(_) => Some(Ordering::Greater),
            },
            Score::NegativeMateIn(nmi) => {
                match other {
                    Score::NegativeMateIn(onmi) => Some(nmi.cmp(onmi)), // We want a higher bad mate
                    _ => Some(Ordering::Less),
                }
            }
        }
    }
}

impl Neg for Score {
    type Output = Score;

    fn neg(self) -> Score {
        match self {
            Score::PositiveMateIn(pmi) => Score::NegativeMateIn(pmi),
            Score::Score(s) => Score::Score(-s),
            Score::NegativeMateIn(nmi) => Score::PositiveMateIn(nmi),
        }
    }
}

#[derive(Debug)]
pub(crate) struct SearchResult {
    pub score: Score,
    pub poisoned: bool, // Don't add to cache - impure considerations affected eval
    #[cfg(debug_assertions)]
    pub backtrace: Backtrace,
    #[cfg(debug_assertions)]
    pub fen: String,
}

impl SearchResult {
    pub fn new(score: Score, poisoned: bool) -> SearchResult {
        #[cfg(debug_assertions)]
        {
            let backtrace = Backtrace::capture();
            SearchResult {
                score,
                poisoned,
                backtrace,
                fen: String::new(),
            }
        }
        #[cfg(not(debug_assertions))]
        SearchResult { score, poisoned }
    }

    #[cfg(debug_assertions)]
    pub fn new_backtrace(
        score: Score,
        poisoned: bool,
        backtrace: Backtrace,
        fen: String,
    ) -> SearchResult {
        SearchResult {
            score,
            poisoned,
            backtrace,
            fen,
        }
    }

    pub fn normal(score: Score) -> SearchResult {
        #[cfg(debug_assertions)]
        {
            let backtrace = Backtrace::capture();
            SearchResult {
                score,
                poisoned: false,
                backtrace,
                fen: String::new(),
            }
        }
        #[cfg(not(debug_assertions))]
        SearchResult {
            score,
            poisoned: false,
        }
    }

    pub fn poisoned(score: Score) -> SearchResult {
        #[cfg(debug_assertions)]
        {
            let backtrace = Backtrace::capture();
            SearchResult {
                score,
                poisoned: true,
                backtrace,
                fen: String::new(),
            }
        }
        #[cfg(not(debug_assertions))]
        SearchResult {
            score,
            poisoned: true,
        }
    }
}

impl Neg for SearchResult {
    type Output = SearchResult;
    fn neg(self) -> SearchResult {
        #[cfg(debug_assertions)]
        {
            SearchResult {
                score: -self.score,
                poisoned: self.poisoned,
                backtrace: self.backtrace,
                fen: self.fen,
            }
        }
        #[cfg(not(debug_assertions))]
        Self {
            score: -self.score,
            poisoned: self.poisoned,
        }
    }
}
