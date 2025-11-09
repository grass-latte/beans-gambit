use std::collections::{HashMap, HashSet};
use std::sync::{OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard};
use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, EnumIter};
use vampirc_uci::UciOptionConfig;

#[derive(Debug, EnumIter, AsRefStr, Eq, PartialEq, Hash, Copy, Clone)]
pub enum UciOptions {
    Example,
}

impl UciOptions {
    pub fn get_type(&self) -> UciOptionConfig {
        match self {
            UciOptions::Example => UciOptionConfig::Button {
                name: "Example".to_string(),
            },
        }
    }

    pub fn from_string<S: AsRef<str>>(s: S) -> Option<UciOptions> {
        let lower = s.as_ref().to_ascii_lowercase();

        UciOptions::iter().find(|&option| option.get_type().get_name() == lower)
    }
}

#[derive(Debug)]
pub struct GlobalState {
    pub debug: bool,
    command_counter: usize,
    commands_in_progress: HashSet<usize>,
    options: HashMap<UciOptions, String>,
}

impl Default for GlobalState {
    fn default() -> Self {
        Self::new()
    }
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

    #[allow(clippy::result_unit_err)]
    pub fn set_option_named<S1: AsRef<str>, S2: AsRef<str>>(
        &mut self,
        option: S1,
        value: S2,
    ) -> Result<UciOptions, ()> {
        let o = UciOptions::from_string(option).ok_or(())?;
        self.set_option(o, value);
        Ok(o)
    }

    #[allow(clippy::result_unit_err)]
    pub fn unset_option_named<S: AsRef<str>>(&mut self, option: S) -> Result<UciOptions, ()> {
        let o = UciOptions::from_string(option).ok_or(())?;
        self.unset_option(o);
        Ok(o)
    }

    pub fn get_option(&self, option: UciOptions) -> Option<String> {
        self.options.get(&option).cloned()
    }
}

static SLOW_GLOBAL_STATE: OnceLock<RwLock<GlobalState>> = OnceLock::new();

pub fn slow_global_state() -> RwLockReadGuard<'static, GlobalState> {
    SLOW_GLOBAL_STATE
        .get_or_init(|| RwLock::new(GlobalState::new()))
        .read()
        .unwrap()
}

pub fn slow_global_state_mut() -> RwLockWriteGuard<'static, GlobalState> {
    SLOW_GLOBAL_STATE
        .get_or_init(|| RwLock::new(GlobalState::new()))
        .write()
        .unwrap()
}
