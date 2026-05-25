mod audio;
mod config;
mod control;
mod hotkey;
mod inject;
mod model;
mod observe;
mod parakeet;
mod postprocess;
mod state;
mod tray;

use std::sync::{Arc, OnceLock};
use tokio::runtime::Runtime;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

pub fn runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime")
    })
}

pub fn spawn<F>(f: F) -> tokio::task::JoinHandle<F::Output>
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    runtime().spawn(f)
}

#[derive(Clone, Copy)]
struct RunMode {
    smoke: bool,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let run_mode = RunMode {
        smoke: args.iter().any(|arg| arg == "--smoke"),
    };

    match args.get(1).map(|s| s.as_str()) {
        Some("--smoke") => {}
        Some("--quit") => {
            match control::request_quit() {
                Ok(()) => println!("transcrust quit signal sent"),
                Err(e) => {
                    eprintln!("Quit failed: {e}");
                    std::process::exit(1);
                }
            }
            return;
        }
        Some("--probe-onnx") => {
            let Some(path) = args.get(2) else {
                eprintln!("Usage: transcrust --probe-onnx /path/to/model.onnx");
                std::process::exit(1);
            };
            run_probe_suite(std::slice::from_ref(&std::path::PathBuf::from(path)));
            return;
        }
        Some("--probe-suite") => {
            let paths: Vec<std::path::PathBuf> = args.iter().skip(2).map(std::path::PathBuf::from).collect();
            if paths.is_empty() {
                eprintln!("Usage: transcrust --probe-suite /path/to/model1.onnx [/path/to/model2.onnx ...]");
                std::process::exit(1);
            }
            run_probe_suite(&paths);
            return;
        }
        Some("--download-model") => {
            let model_name = args.get(2).map(|s| s.as_str());
            let _ = runtime();
            runtime().block_on(async {
                match model::download_model(model_name).await {
                    Ok(path) => println!("Done: {}", path.display()),
                    Err(e) => {
                        eprintln!("Download failed: {e}");
                        std::process::exit(1);
                    }
                }
            });
            return;
        }
        Some("--doctor") => {
            run_doctor();
            return;
        }
        Some("--list-devices") => {
            hotkey::list_devices();
            println!();
            audio::AudioCapture::list_devices();
            return;
        }
        Some("--config") => {
            println!("{}", config::config_path().display());
            return;
        }
        Some("--version") => {
            println!("transcrust {}", env!("CARGO_PKG_VERSION"));
            return;
        }
        Some("--help" | "-h") => {
            println!("transcrust — Observable Parakeet-only push-to-talk voice input");
            println!();
            println!("Usage: transcrust [OPTION]");
            println!();
            println!("Options:");
            println!("  --probe-onnx <PATH>         Probe a single ONNX file across builder variants");
            println!("  --probe-suite <PATH...>     Probe multiple ONNX files across builder variants");
            println!("  --smoke                     Run with terminal phase logging enabled");
            println!("  --quit                      Ask a running transcrust instance to exit");
            println!("  --doctor                    Print phase-relevant environment info");
            println!("  --download-model [MODEL]    Download a Parakeet model");
            println!();
            println!("Available models for --download-model:");
            println!("  parakeet-tdt-0.6b-int8      INT8 quantised, ~250 MB (default)");
            println!("  parakeet-tdt-0.6b-int4      INT4 quantised, ~409 MB (less RAM at inference)");
            println!("  parakeet-tdt-0.6b           FP32 full precision, ~1.4 GB");
            println!();
            println!("Models are saved to ~/.local/share/transcrust/models/");
            println!("  --list-devices              List keyboard and audio devices");
            println!("  --config                    Print config path");
            println!("  --version                   Print version");
            println!("  --help                      Show this help");
            return;
        }
        Some(flag) => {
            eprintln!("Unknown flag: {flag}");
            eprintln!("Run with --help for usage");
            std::process::exit(1);
        }
        None => {}
    }

    init_ort_default();
    let _ = runtime();
    let config = config::load();
    runtime().block_on(run(config, run_mode));
}

fn run_probe_suite(paths: &[std::path::PathBuf]) {
    let observer =
        observe::Observer::new(120, false, true).expect("Failed to initialize probe observer");
    init_ort_for_probe(&observer);
    observer.phase("startup", &format!("log file: {}", observer.log_path().display()));
    for path in paths {
        observer.phase("startup", &format!("probe path: {}", path.display()));
    }

    if let Err(e) = parakeet::run_probe_suite(&observer, paths, std::time::Duration::from_secs(20)) {
        observer.error("probe", &e);
        std::process::exit(1);
    }
}

fn init_ort_default() {
    let _ = ort::init()
        .with_name("transcrust")
        .with_telemetry(false)
        .commit();
}

fn init_ort_for_probe(observer: &observe::Observer) {
    let _ = observer;
    let _ = ort::init()
        .with_name("transcrust")
        .with_telemetry(false)
        .commit();
}

async fn run(config: config::Config, run_mode: RunMode) {
    let observer = observe::Observer::new(
        config.observe.sample_chars,
        config.observe.desktop_notifications,
        run_mode.smoke,
    )
    .expect("Failed to initialize observer");
    observer.phase("startup", &format!("log file: {}", observer.log_path().display()));
    if run_mode.smoke {
        observer.phase("startup", "smoke mode enabled");
    }
    let _pid_guard = match control::write_pid_file() {
        Ok(guard) => Some(guard),
        Err(e) => {
            observer.error("startup", &e);
            None
        }
    };

    let model_path = match model::find_model_path(config.model.path.as_deref()) {
        Some(path) => path,
        None => {
            observer.error("startup", "No Parakeet model found");
            for path in model::explain_search_paths() {
                observer.phase("startup", &format!("searched: {path}"));
            }
            return;
        }
    };

    observer.phase("startup", &format!("model: {}", model_path.display()));
    for result in model::probe_model_files(&model_path) {
        observer.phase("startup.probe", &result);
    }

    let audio = match audio::AudioCapture::new(config.audio.device.as_deref()) {
        Ok(audio) => Arc::new(audio),
        Err(e) => {
            observer.error("audio", &e);
            return;
        }
    };

    let state = Arc::new(state::StateMachine::new());
    spawn(tray::run_tray(
        state.rx.clone(),
        observer.log_path().display().to_string(),
    ));
    let mut hotkey_rx = hotkey::listen(&config.hotkey).await;
    let transcription =
        match parakeet::ParakeetService::new(
            model_path.to_string_lossy().into_owned(),
            config.observe.idle_timeout_secs,
        ) {
            Ok(service) => service,
            Err(e) => {
                observer.error("startup", &e);
                return;
            }
        };
    let mut active_audio_rx: Option<std::sync::mpsc::Receiver<Vec<f32>>> = None;

    loop {
        tokio::select! {
            Some(event) = hotkey_rx.recv() => {
                match event {
                    hotkey::HotkeyEvent::Pressed => {
                        if state.current() == state::AppState::Idle {
                            observer.phase("recording", "hotkey pressed; starting capture");
                            observer.notify("Transcrust", "Recording started");
                            state.transition(state::AppState::Recording);
                            active_audio_rx = Some(audio.start_recording());
                        }
                    }
                    hotkey::HotkeyEvent::Released => {
                        if state.current() == state::AppState::Recording {
                            observer.phase("recording", "hotkey released; stopping capture");
                            audio.stop_recording();
                            state.transition(state::AppState::Transcribing);

                            let Some(audio_rx) = active_audio_rx.take() else {
                                observer.error("recording", "release seen without active audio receiver");
                                state.transition(state::AppState::Error);
                                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                                state.transition(state::AppState::Idle);
                                continue;
                            };

                            let sample_rate = audio.sample_rate();
                            let state = state.clone();
                            let output_cfg = config.output.clone();
                            let transcription = transcription.clone();
                            let observer = observer.clone();

                            spawn(async move {
                                run_transcription_pipeline(
                                    observer,
                                    transcription,
                                    audio_rx,
                                    sample_rate,
                                    output_cfg,
                                    state,
                                ).await;
                            });
                        }
                    }
                }
            }
            _ = tokio::signal::ctrl_c() => {
                observer.phase("shutdown", "received ctrl-c");
                break;
            }
        }
    }
}

async fn run_transcription_pipeline(
    observer: observe::Observer,
    transcription: parakeet::ParakeetService,
    audio_rx: std::sync::mpsc::Receiver<Vec<f32>>,
    sample_rate: u32,
    output_cfg: config::OutputConfig,
    state: Arc<state::StateMachine>,
) {
    observer.phase("transcription", "starting Parakeet transcription");
    let result = match tokio::time::timeout(
        std::time::Duration::from_secs(45),
        transcription.transcribe(observer.clone(), audio_rx, sample_rate),
    )
    .await
    {
        Ok(result) => result,
        Err(_) => {
            observer.error("transcription", "timed out after 45s");
            state.transition(state::AppState::Error);
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            state.transition(state::AppState::Idle);
            return;
        }
    };

    match result {
        Ok(text) if !text.is_empty() => {
            observer.sample("transcription.raw", &text);
            let fixed = postprocess::fix_transcription(&text);
            observer.sample("transcription.postprocess", &fixed);
            state.transition(state::AppState::Injecting);
            observer.phase("inject", "injecting transcript");

            if let Err(e) = inject::inject_text(&fixed, &output_cfg).await {
                observer.error("inject", &e);
                state.transition(state::AppState::Error);
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                state.transition(state::AppState::Idle);
                return;
            }

            observer.notify("Transcrust", &format!("Injected: {}", fixed.chars().take(60).collect::<String>()));
            observer.phase("inject", "inject complete");
            state.transition(state::AppState::Complete);
            tokio::time::sleep(std::time::Duration::from_millis(850)).await;
            state.transition(state::AppState::Idle);
        }
        Ok(_) => {
            observer.phase("transcription", "empty transcript");
            observer.notify("Transcrust", "Empty transcript");
            state.transition(state::AppState::Idle);
        }
        Err(e) => {
            observer.error("transcription", &e);
            state.transition(state::AppState::Error);
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            state.transition(state::AppState::Idle);
        }
    }
}

fn run_doctor() {
    let config = config::load();
    println!("Config path: {}", config::config_path().display());
    println!(
        "Model override: {}",
        config.model.path.as_deref().unwrap_or("<auto>")
    );
    match model::find_model_path(config.model.path.as_deref()) {
        Some(path) => println!("Resolved model: {}", path.display()),
        None => println!("Resolved model: <missing>"),
    }
    println!("Search paths:");
    for path in model::explain_search_paths() {
        println!("  {path}");
    }
    println!("Preferred int8 model dir: {}", model::preferred_int8_model_dir().display());
    println!("Required int8 files:");
    for file in model::required_int8_files() {
        println!("  {file}");
    }
    println!("Quit pid file: {}", control::pid_file_path().display());
    for cmd in ["wtype", "dotool", "notify-send"] {
        let found = std::process::Command::new("sh")
            .arg("-lc")
            .arg(format!("command -v {cmd} >/dev/null"))
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        println!("Command {cmd}: {}", if found { "yes" } else { "no" });
    }
}
