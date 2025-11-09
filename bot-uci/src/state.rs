use std::collections::{HashMap, HashSet};
use std::sync::{OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard};
use strum_macros::{AsRefStr, EnumIter, EnumString};

#[derive(Debug, EnumIter, AsRefStr, EnumString, Eq, PartialEq, Hash, Copy, Clone)]
#[strum(serialize_all = "snake_case", ascii_case_insensitive)]
pub enum UciOptions {
    Example
}

#[derive(Debug)]
pub struct GlobalState {
    pub debug: bool,
    command_counter: usize,
    commands_in_progress: HashSet<usize>,
    options: HashMap<UciOptions, String>,
}

impl GlobalState {
    pub fn new() -> GlobalState {
        GlobalState {
            debug: false,
            command_counter: 0,
            commands_in_progress: HashSet::new(),
            options: HashMap::new(),
        }
    }

    pub fn start_command(&mut self) -> usize {
        self.commands_in_progress.insert(self.command_counter);
        self.command_counter += 1;
        self.command_counter - 1
    }

    pub fn end_command(&mut self, command: usize) {
        self.commands_in_progress.remove(&command);
    }

    pub fn is_ready(&self) -> bool {
        self.commands_in_progress.is_empty()
    }

    pub fn set_option<S: AsRef<str>>(&mut self, option: UciOptions, value: S) {
        self.options.insert(option, value.as_ref().to_string());
    }

    pub fn unset_option(&mut self, option: UciOptions) {
        self.options.remove(&option);
    }

    pub fn set_option_named<S1: AsRef<str>, S2: AsRef<str>>(&mut self, option: S1, value: S2) -> Result<UciOptions, ()> {
        let o = option.as_ref().parse::<UciOptions>().map_err(|_| ())?;
        self.set_option(o, value);
        Ok(o)
    }

    pub fn unset_option_named<S: AsRef<str>>(&mut self, option: S) -> Result<UciOptions, ()> {
        let o = option.as_ref().parse::<UciOptions>().map_err(|_| ())?;
        self.unset_option(o);
        Ok(o)
    }

    pub fn get_option(&self, option: UciOptions) -> Option<String> {
        self.options.get(&option).cloned()
    }
}

static SLOW_GLOBAL_STATE: OnceLock<RwLock<GlobalState>> = OnceLock::new();

pub fn slow_global_state() -> RwLockReadGuard<'static, GlobalState> {
    SLOW_GLOBAL_STATE.get_or_init(|| RwLock::new(GlobalState::new())).read().unwrap()
}

pub fn slow_global_state_mut() -> RwLockWriteGuard<'static, GlobalState> {
    SLOW_GLOBAL_STATE.get_or_init(|| RwLock::new(GlobalState::new())).write().unwrap()
}
