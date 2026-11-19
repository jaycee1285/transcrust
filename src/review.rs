//! Desktop adapter. The daemon launches the window in a child so the GUI owns
//! its main thread. Only JSON crosses stdin/stdout; no extra model is loaded.
use crate::correction::{Change, Dictation, Pair};
use eframe::egui;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[path = "review_theme.rs"]
mod theme;

#[derive(Serialize, Deserialize)]
pub struct Request {
    pub dictation: Dictation,
    pub vocabulary_path: PathBuf,
    pub notice: Option<String>,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum Action {
    Replace,
    Copy,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Response {
    pub text: String,
    pub action: Action,
    #[serde(default)]
    pub kept: Vec<Pair>,
}

pub async fn open(request: Request) -> Result<Option<Response>, String> {
    use tokio::io::AsyncWriteExt;
    let mut command =
        tokio::process::Command::new(std::env::current_exe().map_err(|e| e.to_string())?);
    // The Nix wrapper supplies a Mesa driver built against our glibc. Loading
    // /run/opengl-driver instead can fail after the host upgrades. Scope this
    // override to the GUI child; external desktop tools keep their own driver.
    if let Some(vendor) = std::env::var_os("TRANSCRUST_EGL_VENDOR_LIBRARY_FILENAMES") {
        command.env("__EGL_VENDOR_LIBRARY_FILENAMES", vendor);
    }
    let mut child = command
        .arg("--review-window")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("Could not open correction window: {e}"))?;
    let input = serde_json::to_vec(&request).map_err(|e| e.to_string())?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(&input).await.map_err(|e| e.to_string())?;
    }
    let output = child.wait_with_output().await.map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(format!("Correction window exited with {}", output.status));
    }
    if output.stdout.is_empty() {
        return Ok(None);
    }
    serde_json::from_slice(&output.stdout).map_err(|e| format!("Invalid correction response: {e}"))
}

pub fn run_window() -> Result<(), String> {
    let request: Request = serde_json::from_reader(std::io::stdin()).map_err(|e| e.to_string())?;
    let result = Arc::new(Mutex::new(None));
    let result_for_app = result.clone();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_app_id("transcrust-review")
            .with_inner_size([640.0, 520.0])
            .with_min_inner_size([420.0, 360.0]),
        centered: true, // Wayland placement is owned by the compositor.
        ..Default::default()
    };
    eframe::run_native(
        "Correct last dictation",
        options,
        Box::new(move |cc| {
            cc.egui_ctx.set_theme(egui::ThemePreference::System);
            theme::apply(&cc.egui_ctx);
            let mut style = (*cc.egui_ctx.style()).clone();
            style
                .text_styles
                .insert(egui::TextStyle::Body, egui::FontId::proportional(17.0));
            style
                .text_styles
                .insert(egui::TextStyle::Button, egui::FontId::proportional(16.0));
            style.spacing.item_spacing = egui::vec2(10.0, 10.0);
            cc.egui_ctx.set_style(style);
            let text = request
                .dictation
                .corrected
                .clone()
                .unwrap_or_else(|| request.dictation.delivered.clone());
            Ok(Box::new(Editor {
                request,
                text,
                first_frame: true,
                caret_confirmed: false,
                pending: None,
                pairs: Vec::new(),
                error: None,
                result: result_for_app,
            }))
        }),
    )
    .map_err(|e| e.to_string())?;
    if let Some(response) = result.lock().unwrap().take() {
        serde_json::to_writer(std::io::stdout(), &response).map_err(|e| e.to_string())?;
    }
    Ok(())
}

struct Editor {
    request: Request,
    text: String,
    first_frame: bool,
    caret_confirmed: bool,
    pending: Option<Action>,
    pairs: Vec<(Change, Pair, bool)>,
    error: Option<String>,
    result: Arc<Mutex<Option<Response>>>,
}

impl Editor {
    fn begin(&mut self, action: Action, ctx: &egui::Context) {
        self.pairs = crate::correction::changes(&self.request.dictation.delivered, &self.text)
            .into_iter()
            .map(|change| {
                let pair = change.pair();
                (change, pair, false)
            })
            .collect();
        self.pending = Some(action);
        if self.pairs.is_empty() {
            self.finish(ctx);
        }
    }
    fn finish(&mut self, ctx: &egui::Context) {
        let kept: Vec<Pair> = self
            .pairs
            .iter()
            .filter(|(_, _, keep)| *keep)
            .map(|(_, p, _)| p.clone())
            .collect();
        match transcrust_core::vocabulary::save_pairs(&self.request.vocabulary_path, &kept) {
            Ok(()) => {
                *self.result.lock().unwrap() = Some(Response {
                    text: self.text.clone(),
                    action: self.pending.unwrap(),
                    kept,
                });
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            Err(e) => {
                self.error = Some(format!(
                "Could not save vocabulary: {e}. Drop the proposals to continue without saving."
            ))
            }
        }
    }
}

impl eframe::App for Editor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        let submit = ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Enter));
        egui::CentralPanel::default().frame(egui::Frame::central_panel(&ctx.style()).inner_margin(24.0)).show(ctx, |ui| {
            ui.heading(if self.pending.is_some() { "Review your changes" } else { "Correct last dictation" });
            ui.label(format!("{} · {:.1}s audio · {:.1}s wait", self.request.dictation.mode,
                self.request.dictation.audio_secs, self.request.dictation.wait_secs));
            ui.add_space(8.0);
            if let Some(notice) = &self.request.notice {
                ui.colored_label(ui.visuals().error_fg_color, notice);
            }
            if self.pending.is_some() {
                ui.label(format!("{} changes. All edits will be applied and saved.", self.pairs.len()));
                ui.label("Only check Learn for a name or phrase you want the glossary to remember.");
                egui::ScrollArea::vertical().max_height((ui.available_height() - 150.0).max(80.0)).show(ui, |ui| {
                    for (index, (change, pair, keep)) in self.pairs.iter_mut().enumerate() {
                        ui.push_id(index, |ui| {
                            ui.separator();
                            ui.label(if change.before.is_empty() { "Added" } else if change.after.is_empty() { "Removed" } else { "Changed" });
                            if !change.before.is_empty() { ui.label(egui::RichText::new(&change.before).strikethrough()); }
                            if !change.after.is_empty() { ui.label(egui::RichText::new(&change.after).strong()); }
                            ui.checkbox(keep, "Learn a name or phrase from this change");
                            if *keep {
                                ui.label("Heard"); ui.text_edit_singleline(&mut pair.heard);
                                ui.label("Prefer"); ui.text_edit_singleline(&mut pair.canonical);
                                if !crate::correction::valid_pair(pair) {
                                    let reason = if crate::correction::grammar_rewrite(pair) { "This is a sentence edit, not a vocabulary name. Uncheck Learn; your text correction will still be saved." } else { "Choose a phrase on each side: up to 12 words, without commas or sentence punctuation. Uncheck Learn to apply only the text edit." };
                                    ui.colored_label(ui.visuals().error_fg_color, reason);
                                }
                            }
                        });
                    }
                });
                if let Some(error) = &self.error { ui.colored_label(ui.visuals().error_fg_color, error); }
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    let label = match self.pending.unwrap() { Action::Replace => "Apply changes & replace", Action::Copy => "Apply changes & copy" };
                    let valid = self.pairs.iter().all(|(_, pair, keep)| !*keep || crate::correction::valid_pair(pair));
                    if ui.add_enabled(valid, egui::Button::new(label)).clicked() || (submit && valid) { self.finish(ctx); }
                    if ui.button("Back").clicked() { self.pending = None; self.error = None; }
                    if ui.button("Cancel").clicked() { ctx.send_viewport_cmd(egui::ViewportCommand::Close); }
                });
            } else {
                egui::ScrollArea::vertical().max_height((ui.available_height() - 190.0).max(100.0)).show(ui, |ui| {
                    let mut output = egui::TextEdit::multiline(&mut self.text)
                        .desired_width(f32::INFINITY).desired_rows(8).show(ui);
                    if self.first_frame {
                        output.response.request_focus();
                        output.state.cursor.set_char_range(Some(egui::text::CCursorRange::two(
                            egui::text::CCursor::new(0), egui::text::CCursor::new(self.text.chars().count()))));
                        output.state.store(ctx, output.response.id);
                        self.first_frame = false;
                    }
                    ui.collapsing("Original text", |ui| {
                        ui.label(&self.request.dictation.delivered);
                        ui.collapsing("Raw recognition", |ui| { ui.label(&self.request.dictation.raw); });
                    });
                });
                ui.add_space(8.0);
                if self.request.dictation.replaceable {
                    ui.checkbox(&mut self.caret_confirmed, "The caret is still just after this dictation in my editor");
                } else {
                    ui.label("No confirmed typed span to replace. Copy the correction and paste it yourself.");
                }
                ui.horizontal(|ui| {
                    let replace = self.request.dictation.replaceable && self.caret_confirmed;
                    if ui.add_enabled(replace, egui::Button::new("Review & replace")).clicked() || (submit && replace) {
                        self.begin(Action::Replace, ctx);
                    }
                    if ui.add_enabled(!self.text.is_empty(), egui::Button::new("Copy corrected text")).clicked() {
                        self.begin(Action::Copy, ctx);
                    }
                    if ui.button("Cancel").clicked() { ctx.send_viewport_cmd(egui::ViewportCommand::Close); }
                });
                ui.small("Ctrl+Enter replaces · Enter adds a line · Esc cancels");
            }
        });
    }
}

/// Only the latest record is retained, without audio. Restarting preserves the
/// correction but invalidates the destination/caret assumption.
fn last_path() -> std::path::PathBuf {
    dirs::state_dir()
        .unwrap_or_else(|| dirs::data_local_dir().unwrap_or_else(|| ".".into()))
        .join("transcrust")
        .join("last-dictation.json")
}

pub fn load_last() -> Option<Dictation> {
    let mut record: Dictation = serde_json::from_slice(&std::fs::read(last_path()).ok()?).ok()?;
    record.replaceable = false;
    Some(record)
}

pub fn save_last(record: &Dictation) -> Result<(), String> {
    save_last_at(&last_path(), record)
}

fn save_last_at(path: &std::path::Path, record: &Dictation) -> Result<(), String> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
    let history = path.parent().unwrap().join("history");
    std::fs::create_dir_all(&history).map_err(|e| e.to_string())?;
    // Archive legacy latest-only state before the next take can overwrite it.
    if let Ok(previous) = std::fs::read(&path) {
        if let Ok(old) = serde_json::from_slice::<Dictation>(&previous) {
            if old.id.is_empty() {
                let legacy = history.join(format!("legacy-{}.json", crate::corpus::stamp()));
                if !legacy.exists() {
                    write_private(&legacy, &previous)?;
                }
            }
        }
    }
    if !record.id.is_empty() {
        let archived = history.join(format!("{}.json", record.id));
        let tmp = archived.with_extension("tmp");
        write_private(
            &tmp,
            &serde_json::to_vec_pretty(record).map_err(|e| e.to_string())?,
        )?;
        std::fs::rename(tmp, archived).map_err(|e| e.to_string())?;
    }
    let tmp = path.with_extension("tmp");
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(&tmp)
        .map_err(|e| e.to_string())?;
    file.write_all(&serde_json::to_vec_pretty(record).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())?;
    std::fs::rename(tmp, path).map_err(|e| e.to_string())
}

fn write_private(path: &std::path::Path, bytes: &[u8]) -> Result<(), String> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)
        .map_err(|e| e.to_string())?;
    file.write_all(bytes).map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())
}

fn save_correction(record: &Dictation, response: &Response) -> Result<(), String> {
    append_correction(
        &last_path().parent().unwrap().join("corrections.jsonl"),
        record,
        response,
    )?;
    crate::corpus::save_correction(&record.id, &response.text)
}

fn append_correction(
    path: &std::path::Path,
    record: &Dictation,
    response: &Response,
) -> Result<(), String> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let event = serde_json::json!({
        "recorded": crate::corpus::stamp(), "dictation": record,
        "changes": crate::correction::changes(&record.delivered, &response.text),
        "kept": response.kept, "action": response.action,
    });
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .mode(0o600)
        .open(path)
        .map_err(|e| e.to_string())?;
    writeln!(file, "{event}").map_err(|e| e.to_string())?;
    file.sync_data().map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod history_tests {
    use super::*;
    #[test]
    fn new_takes_retain_previous_text_and_every_correction() {
        let dir =
            std::env::temp_dir().join(format!("transcrust-history-{}", crate::corpus::take_id()));
        let last = dir.join("last-dictation.json");
        let mut take = Dictation {
            id: "take-one".into(),
            raw: "flower".into(),
            delivered: "flower".into(),
            corrected: None,
            buffer_text: "flower".into(),
            mode: "test".into(),
            audio_secs: 1.0,
            wait_secs: 0.1,
            replaceable: true,
        };
        save_last_at(&last, &take).unwrap();
        for text in ["Flo", "Flo and Mater."] {
            let response = Response {
                text: text.into(),
                action: Action::Copy,
                kept: vec![],
            };
            take.corrected = Some(text.into());
            append_correction(&dir.join("corrections.jsonl"), &take, &response).unwrap();
            save_last_at(&last, &take).unwrap();
        }
        take.id = "take-two".into();
        take.raw = "next recording".into();
        take.corrected = None;
        save_last_at(&last, &take).unwrap();
        let old: Dictation =
            serde_json::from_slice(&std::fs::read(dir.join("history/take-one.json")).unwrap())
                .unwrap();
        assert_eq!(old.raw, "flower");
        assert_eq!(old.corrected.as_deref(), Some("Flo and Mater."));
        assert_eq!(old.buffer_text, "flower"); // Copy must not change destination state.
        let events = std::fs::read_to_string(dir.join("corrections.jsonl"))
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0]["dictation"]["corrected"], "Flo");
        assert_eq!(events[1]["dictation"]["corrected"], "Flo and Mater.");
        assert!(events
            .iter()
            .all(|event| event["kept"].as_array().unwrap().is_empty()));
        std::fs::remove_dir_all(dir).unwrap();
    }
}

/// Desktop orchestration; engine/correction data stays independent of it.
/// A failed replacement reopens with the edited text and an explicit recovery,
/// even when desktop notifications are disabled.
pub async fn correct_last(
    observer: crate::observe::Observer,
    state: Arc<crate::state::StateMachine>,
    last: Arc<Mutex<Option<Dictation>>>,
    mut dictation: Dictation,
) {
    let mut notice = None;
    loop {
        observer.phase("correct", "opening last dictation");
        state.transition(crate::state::AppState::Reviewing);
        let request = Request {
            dictation: dictation.clone(),
            vocabulary_path: transcrust_core::vocabulary::path(),
            notice: notice.take(),
        };
        match open(request).await {
            Ok(Some(response)) => {
                dictation.corrected = Some(response.text.clone());
                if let Err(error) = save_correction(&dictation, &response) {
                    observer.error("correct.history", &error);
                    notice = Some(format!("Your correction could not be archived: {error}. It has not been typed. Try again or copy the text."));
                    continue;
                }
                observer.phase(
                    "correct.history",
                    &format!(
                        "saved {} edits for take {}",
                        crate::correction::changes(&dictation.delivered, &response.text).len(),
                        dictation.id
                    ),
                );
                if matches!(response.action, Action::Replace) {
                    dictation.replaceable = false;
                }
                // Persist before automation; failure must not lose the edit.
                if let Err(e) = save_last(&dictation) {
                    observer.error("correct.save", &e);
                }
                *last.lock().unwrap() = Some(dictation.clone());
                let result = match response.action {
                    Action::Copy => crate::inject::copy_correction(&response.text),
                    Action::Replace => {
                        state.transition(crate::state::AppState::Injecting);
                        crate::inject::replace_last(
                            &dictation.buffer_text,
                            &response.text,
                            &observer,
                        )
                        .await
                    }
                };
                match result {
                    Ok(()) => {
                        if matches!(response.action, Action::Replace) {
                            dictation.buffer_text = response.text;
                            dictation.replaceable = !dictation.buffer_text.is_empty();
                            observer.phase("correct", "corrected text replaced");
                        } else {
                            observer.phase("correct", "corrected text copied");
                        }
                        if let Err(e) = save_last(&dictation) {
                            observer.error("correct.save", &e);
                        }
                        *last.lock().unwrap() = Some(dictation);
                        break;
                    }
                    Err(e) => {
                        observer.error("correct", &e);
                        notice = Some(format!("Could not deliver the correction: {e}. Your edited text is retained. Use Copy, or select and copy it here."));
                    }
                }
            }
            Ok(None) => {
                observer.phase("correct", "cancelled");
                break;
            }
            Err(e) => {
                observer.error("correct", &e);
                break;
            }
        }
    }
    state.transition(crate::state::AppState::Idle);
}
