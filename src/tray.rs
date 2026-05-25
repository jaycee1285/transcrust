use ksni::{self, menu::StandardItem, MenuItem, ToolTip};
use tokio::sync::watch;

use crate::state::AppState;

struct TranscrustTray {
    state: AppState,
    log_path: String,
}

impl ksni::Tray for TranscrustTray {
    fn id(&self) -> String {
        "transcrust".into()
    }

    fn title(&self) -> String {
        "Transcrust".into()
    }

    fn icon_name(&self) -> String {
        match self.state {
            AppState::Idle => "media-playback-start-symbolic".into(),
            AppState::Recording => "media-record-symbolic".into(),
            AppState::Transcribing => "content-loading-symbolic".into(),
            AppState::Injecting => "emblem-ok-symbolic".into(),
            AppState::Complete => "emblem-ok-symbolic".into(),
            AppState::Error => "dialog-error-symbolic".into(),
        }
    }

    fn tool_tip(&self) -> ToolTip {
        ToolTip {
            title: match self.state {
                AppState::Idle => "Transcrust: Ready".into(),
                AppState::Recording => "Transcrust: Recording".into(),
                AppState::Transcribing => "Transcrust: Processing".into(),
                AppState::Injecting => "Transcrust: Injecting".into(),
                AppState::Complete => "Transcrust: Complete".into(),
                AppState::Error => "Transcrust: Error".into(),
            },
            description: self.log_path.clone(),
            icon_name: String::new(),
            icon_pixmap: vec![],
        }
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        vec![MenuItem::Standard(StandardItem {
            label: "Exit".into(),
            activate: Box::new(|_| std::process::exit(0)),
            ..Default::default()
        })]
    }
}

pub async fn run_tray(mut state_rx: watch::Receiver<AppState>, log_path: String) {
    let service = ksni::TrayService::new(TranscrustTray {
        state: AppState::Idle,
        log_path,
    });
    let handle = service.handle();
    service.spawn();

    while state_rx.changed().await.is_ok() {
        let state = *state_rx.borrow();
        handle.update(|tray: &mut TranscrustTray| {
            tray.state = state;
        });
    }
}
