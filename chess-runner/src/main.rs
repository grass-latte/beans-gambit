mod options;
mod prerequisites;
mod run;

use crate::options::select_options;
use prerequisites::prerequisites;
use run::run;

fn main() {
    let engine_path = prerequisites();

    let options = select_options();

    run(options, engine_path);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::run::{ChessOptions, MatchSetup};

    #[test]
    fn test_fastchess_compliance() {
        let engine_path = prerequisites();
        let options = ChessOptions::new(MatchSetup::Compliance);
        run(options, engine_path);
    }
}
