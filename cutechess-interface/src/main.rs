mod options;
mod prerequisites;
mod run;

use crate::options::select_options;
use prerequisites::prerequisites;
use run::run;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum::IntoEnumIterator;

fn main() {
    let engine_path = prerequisites();

    let options = select_options();

    run(options, engine_path);
}
