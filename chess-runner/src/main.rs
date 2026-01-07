mod options;
mod prerequisites;
mod run;

use crate::options::select_options;
use prerequisites::prerequisites;
use run::run;

fn main() {
    let options = select_options();

    run(options, engine_path);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::run::{ChessOptions, MatchType};

    #[test]
    fn test_fastchess_compliance() {
        let engine_path = prerequisites();
        let options = ChessOptions::new(MatchType::Compliance);
        run(options, engine_path);
    }

    #[test]
    fn test_bot_vs_bot() {
        let engine_path = prerequisites();
        let options = ChessOptions::new(MatchType::BeansVsBeans);
        run(options, engine_path);
    }
}
