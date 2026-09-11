use ksni::{
    self,
    menu::{RadioGroup, RadioItem, StandardItem, SubMenu},
    MenuItem, ToolTip,
};
use tokio::sync::{mpsc, watch};

use crate::state::AppState;

struct TranscrustTray {
    state: AppState,
    log_path: String,
    /// One label per discovered model, in `model::discover_models` order.
    engines: Vec<String>,
    /// Index into `engines` that is currently loaded.
    active: usize,
    /// Switch requests go to the main loop, which owns the service and is the
    /// only place allowed to swap it. The tray never builds a model itself.
    request_tx: mpsc::UnboundedSender<usize>,
}

impl ksni::Tray for TranscrustTray {
    fn id(&self) -> String {
        "transcrust".into()
    }

    fn title(&self) -> String {
        "Transcrust".into()
    }

    /// Primary icon source. Hosts recolor themed symbolic icons to match the
    /// panel, which a bitmap cannot do, so a name is still preferable *when the
    /// host's theme actually has it*. Every name here is verified present in
    /// Adwaita — `emblem-ok-symbolic`, used previously for Injecting and
    /// Complete, is not in Adwaita and silently rendered as a generic
    /// placeholder. See [`crate::trayicon`] for the fallback that covers the
    /// themes we cannot check.
    fn icon_name(&self) -> String {
        match self.state {
            AppState::Idle => "media-playback-start-symbolic".into(),
            AppState::Recording => "media-record-symbolic".into(),
            AppState::Transcribing => "content-loading-symbolic".into(),
            AppState::Injecting => "object-select-symbolic".into(),
            AppState::Complete => "object-select-symbolic".into(),
            AppState::Error => "dialog-error-symbolic".into(),
        }
    }

    /// Theme-independent floor. A host that cannot resolve `icon_name` falls
    /// back to these rather than to a generic placeholder.
    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        crate::trayicon::for_state(self.state)
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
            description: match self.engines.get(self.active) {
                Some(engine) => format!("{engine}\n{}", self.log_path),
                None => self.log_path.clone(),
            },
            icon_name: String::new(),
            icon_pixmap: vec![],
        }
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let mut items: Vec<MenuItem<Self>> = Vec::new();

        // With one model installed there is nothing to choose between, so the
        // submenu only appears when a switch is actually possible.
        if self.engines.len() > 1 {
            items.push(MenuItem::SubMenu(SubMenu {
                label: "Engine".into(),
                submenu: vec![MenuItem::RadioGroup(RadioGroup {
                    selected: self.active,
                    select: Box::new(|tray: &mut TranscrustTray, index| {
                        // ksni re-renders the menu from `menu()` the moment this
                        // returns (`update_immediately`), so the radio has to
                        // move here. Leaving it to the main loop's confirmation
                        // paints the *old* engine first and reads as the click
                        // being ignored. The confirmation still arrives on the
                        // watch channel and puts this back if the load fails.
                        tray.active = index;
                        let _ = tray.request_tx.send(index);
                    }),
                    options: self
                        .engines
                        .iter()
                        .map(|label| RadioItem {
                            label: label.clone(),
                            ..Default::default()
                        })
                        .collect(),
                })],
                ..Default::default()
            }));
            items.push(MenuItem::Separator);
        }

        items.push(MenuItem::Standard(StandardItem {
            label: "Exit".into(),
            activate: Box::new(|_| std::process::exit(0)),
            ..Default::default()
        }));
        items
    }
}

pub async fn run_tray(
    mut state_rx: watch::Receiver<AppState>,
    log_path: String,
    engines: Vec<String>,
    mut active_rx: watch::Receiver<usize>,
    request_tx: mpsc::UnboundedSender<usize>,
) {
    let service = ksni::TrayService::new(TranscrustTray {
        state: AppState::Idle,
        log_path,
        engines,
        active: *active_rx.borrow(),
        request_tx,
    });
    let handle = service.handle();
    // ksni requests `org.kde.StatusNotifierItem-<pid>-1` itself, calls
    // RegisterStatusNotifierItem, and re-registers on the watcher's
    // NameOwnerChanged. A manual `gdbus` re-register used to run here; it was
    // racing a registration that had already happened and spawning up to ten
    // subprocesses per launch to do it. Verified redundant by disabling it and
    // confirming the item still lands on the watcher.
    service.spawn();

    // `active_rx` is driven by the main loop after a switch succeeds, so a
    // failed switch leaves the radio pointing at the model that is really live.
    loop {
        tokio::select! {
            changed = state_rx.changed() => {
                if changed.is_err() {
                    break;
                }
                let state = *state_rx.borrow();
                handle.update(|tray: &mut TranscrustTray| {
                    tray.state = state;
                });
            }
            changed = active_rx.changed() => {
                if changed.is_err() {
                    break;
                }
                let active = *active_rx.borrow();
                handle.update(|tray: &mut TranscrustTray| {
                    tray.active = active;
                });
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    // `menu` and `tool_tip` are trait methods.
    use ksni::Tray;

    fn tray(engines: &[&str], active: usize) -> (TranscrustTray, mpsc::UnboundedReceiver<usize>) {
        let (request_tx, request_rx) = mpsc::unbounded_channel();
        let tray = TranscrustTray {
            state: AppState::Idle,
            log_path: "/tmp/transcrust.log".into(),
            engines: engines.iter().map(|name| name.to_string()).collect(),
            active,
            request_tx,
        };
        (tray, request_rx)
    }

    /// Pull the engine RadioGroup out of the rendered menu.
    fn radio_group(tray: &TranscrustTray) -> Option<RadioGroup<TranscrustTray>> {
        for item in tray.menu() {
            if let MenuItem::SubMenu(sub) = item {
                for child in sub.submenu {
                    if let MenuItem::RadioGroup(group) = child {
                        return Some(group);
                    }
                }
            }
        }
        None
    }

    #[test]
    fn selecting_an_engine_moves_the_radio_immediately() {
        // ksni re-renders from `menu()` as soon as the select callback returns,
        // so the callback itself must move `active`. Leaving it to the main
        // loop's confirmation repainted the previous engine and looked like the
        // click had been ignored.
        let (mut tray, mut requests) = tray(&["Parakeet TDT (int4)", "Granite (int8)"], 0);
        let group = radio_group(&tray).expect("engine submenu should exist");
        assert_eq!(group.selected, 0);

        (group.select)(&mut tray, 1);

        assert_eq!(tray.active, 1, "radio must not snap back to the old engine");
        assert_eq!(requests.try_recv(), Ok(1), "main loop must see the request");
        assert_eq!(radio_group(&tray).unwrap().selected, 1);
    }

    #[test]
    fn a_declined_switch_can_be_reverted_by_the_main_loop() {
        let (mut tray, _requests) = tray(&["Parakeet TDT (int4)", "Granite (int8)"], 0);
        let group = radio_group(&tray).expect("engine submenu should exist");
        (group.select)(&mut tray, 1);
        assert_eq!(tray.active, 1);

        // What `run_tray` does when the main loop reports the load failed or
        // was refused: put the radio back on the engine that is really live.
        tray.active = 0;
        assert_eq!(radio_group(&tray).unwrap().selected, 0);
    }

    #[test]
    fn a_single_engine_offers_no_switcher() {
        let (tray, _requests) = tray(&["Parakeet TDT (int4)"], 0);
        assert!(radio_group(&tray).is_none());
        assert!(tray
            .menu()
            .iter()
            .any(|item| matches!(item, MenuItem::Standard(_))));
    }

    #[test]
    fn tooltip_names_the_active_engine() {
        let (tray, _requests) = tray(&["Parakeet TDT (int4)", "Granite (int8)"], 1);
        assert!(tray.tool_tip().description.starts_with("Granite (int8)"));
    }
}
