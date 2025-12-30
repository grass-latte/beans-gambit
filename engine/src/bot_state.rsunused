use std::sync::{OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard};

#[derive(Debug)]
pub struct GlobalState {}

impl Default for GlobalState {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalState {
    pub fn new() -> GlobalState {
        GlobalState {}
    }
}

static SLOW_GLOBAL_STATE: OnceLock<RwLock<GlobalState>> = OnceLock::new();

pub fn slow_bot_state() -> RwLockReadGuard<'static, GlobalState> {
    SLOW_GLOBAL_STATE
        .get_or_init(|| RwLock::new(GlobalState::new()))
        .read()
        .unwrap()
}

pub fn slow_bot_state_mut() -> RwLockWriteGuard<'static, GlobalState> {
    SLOW_GLOBAL_STATE
        .get_or_init(|| RwLock::new(GlobalState::new()))
        .write()
        .unwrap()
}
