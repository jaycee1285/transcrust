use tokio::sync::watch;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    Idle,
    Recording,
    Transcribing,
    Injecting,
    Complete,
    Error,
}

pub struct StateMachine {
    tx: watch::Sender<AppState>,
    pub rx: watch::Receiver<AppState>,
}

impl StateMachine {
    pub fn new() -> Self {
        let (tx, rx) = watch::channel(AppState::Idle);
        Self { tx, rx }
    }

    pub fn transition(&self, to: AppState) {
        let _ = self.tx.send(to);
    }

    pub fn current(&self) -> AppState {
        *self.tx.borrow()
    }
}
