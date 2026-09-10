mod audio;
mod config;
mod control;
mod corpus;
mod dictionary;
mod hotkey;
mod granite;
mod inject;
mod mode;
mod model;
mod moonshine;
mod normalise;
mod observe;
mod parakeet;
mod parakeet_ort;
mod postprocess;
mod state;
mod tray;
mod trayicon;
mod transcription;
mod wav;

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
    /// Toggle-driven dictation for long-form: `--toggle` starts and stops
    /// instead of a key being held down. Off by default; the hold-to-talk path
    /// is untouched by this flag.
    long: bool,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let run_mode = RunMode {
        smoke: args.iter().any(|arg| arg == "--smoke"),
        long: args.iter().any(|arg| arg == "--long"),
    };

    match args.get(1).map(|s| s.as_str()) {
        Some("--smoke") => {}
        // Falls through to the daemon startup below, the same way --smoke does.
        Some("--long") => {}
        Some("--toggle") => {
            match control::request_toggle() {
                Ok(()) => println!("transcrust toggle signal sent"),
                Err(e) => {
                    eprintln!("Toggle failed: {e}");
                    std::process::exit(1);
                }
            }
            return;
        }
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
        Some("--granite-smoke") => {
            let Some(path) = args.get(2) else {
                eprintln!("Usage: transcrust --granite-smoke /path/to/granite-model-dir");
                std::process::exit(1);
            };
            init_ort_default();
            match model::granite_onnx_path(std::path::Path::new(path)) {
                Some(graph) => println!("Granite model: {}", graph.display()),
                None => {
                    eprintln!("No Granite ONNX graph in {path}");
                    std::process::exit(1);
                }
            }
            match granite::run_model_smoke(std::path::Path::new(path)) {
                Ok(text) => {
                    // Run the shared post-transcription pipeline here too, so this
                    // path shows the same text the live hotkey path would inject.
                    println!("Granite smoke passed; synthetic-audio transcript: {text:?}");
                    println!("  post-processed: {:?}", postprocess::fix_transcription(&text));
                }
                Err(error) => {
                    eprintln!("Granite smoke failed: {error}");
                    std::process::exit(1);
                }
            }
            return;
        }
        Some("--record") => {
            // Capture through the same cpal device the hotkey path uses, so a
            // clip banked here is byte-comparable with one banked live.
            let _ = runtime();
            runtime().block_on(run_record());
            return;
        }
        Some("--bench") => {
            let paths: Vec<std::path::PathBuf> =
                args.iter().skip(2).map(std::path::PathBuf::from).collect();
            if paths.is_empty() {
                eprintln!("Usage: transcrust --bench audio.wav [audio2.wav ...]");
                std::process::exit(1);
            }
            init_ort_default();
            let _ = runtime();
            runtime().block_on(run_bench(&paths));
            return;
        }
        Some("--wav") => {
            // Offline twin of the hotkey path: same engine seam, same
            // post-processing, no injection. Writes `<name>.md` beside each WAV.
            let mut paths: Vec<std::path::PathBuf> = Vec::new();
            let mut mode_filter: Option<String> = None;
            let mut rest = args.iter().skip(2);
            while let Some(arg) = rest.next() {
                if arg == "--mode" {
                    let Some(value) = rest.next() else {
                        eprintln!("--mode needs a value, e.g. --mode granite");
                        std::process::exit(1);
                    };
                    mode_filter = Some(value.clone());
                } else {
                    paths.push(std::path::PathBuf::from(arg));
                }
            }
            if paths.is_empty() {
                eprintln!("Usage: transcrust --wav audio.wav [audio2.wav ...] [--mode <substring>]");
                std::process::exit(1);
            }
            init_ort_default();
            let _ = runtime();
            let config = config::load();
            runtime().block_on(wav::run(&paths, mode_filter.as_deref(), &config));
            return;
        }
        Some("--normalise") => {
            // Text in, text out. Reads stdin so it composes with `--wav` output
            // and with anything else that produces a transcript.
            let dir = args.get(2).cloned().unwrap_or_else(|| {
                dirs::data_dir()
                    .unwrap_or_default()
                    .join("transcrust/models/s1-mini-onnx")
                    .to_string_lossy()
                    .into_owned()
            });
            let style = normalise::Style {
                structure: if args.iter().any(|a| a == "--lists") { "lists" } else { "prose" },
                ..normalise::Style::default()
            };
            init_ort_default();
            let started = std::time::Instant::now();
            let mut model = match normalise::Normaliser::load(std::path::Path::new(&dir)) {
                Ok(model) => model,
                Err(error) => { eprintln!("{error}"); std::process::exit(1); }
            };
            eprintln!("cold load: {:.2}s", started.elapsed().as_secs_f64());

            let mut input = String::new();
            if let Err(error) = std::io::Read::read_to_string(&mut std::io::stdin(), &mut input) {
                eprintln!("failed to read stdin: {error}");
                std::process::exit(1);
            }
            let started = std::time::Instant::now();
            match model.normalise(input.trim(), style) {
                Ok(text) => {
                    let elapsed = started.elapsed().as_secs_f64();
                    let words = text.split_whitespace().count();
                    eprintln!("{words} words in {elapsed:.2}s");
                    println!("{text}");
                }
                Err(error) => { eprintln!("{error}"); std::process::exit(1); }
            }
            return;
        }
        Some("--moonshine") => {
            // Evaluation surface, not yet an engine: the question is cold load
            // time against Parakeet and Granite on the same clips.
            let Some(dir) = args.get(2) else {
                eprintln!("Usage: transcrust --moonshine /path/to/moonshine-dir <audio.wav>");
                std::process::exit(1);
            };
            let Some(wav) = args.get(3) else {
                eprintln!("Usage: transcrust --moonshine /path/to/moonshine-dir <audio.wav>");
                std::process::exit(1);
            };
            init_ort_default();
            let started = std::time::Instant::now();
            let mut model = match moonshine::LoadedMoonshine::load(std::path::Path::new(dir)) {
                Ok(model) => model,
                Err(error) => { eprintln!("{error}"); std::process::exit(1); }
            };
            println!("cold load: {:.2}s", started.elapsed().as_secs_f64());
            let (samples, rate) = match audio::read_wav_mono(std::path::Path::new(wav)) {
                Ok(pair) => pair,
                Err(error) => { eprintln!("{error}"); std::process::exit(1); }
            };
            let seconds = samples.len() as f64 / rate as f64;
            let audio_16k = audio::resample_to_16k(&samples, rate);
            let started = std::time::Instant::now();
            match model.transcribe(&audio_16k) {
                Ok(text) => {
                    let elapsed = started.elapsed().as_secs_f64();
                    println!("audio {seconds:.2}s  wall {elapsed:.2}s  RTF {:.3}", elapsed / seconds);
                    println!("text: {text:?}");
                    println!("post-processed: {:?}", postprocess::fix_transcription(&text));
                }
                Err(error) => { eprintln!("{error}"); std::process::exit(1); }
            }
            return;
        }
        Some("--parakeet-direct") => {
            // Drives encoder + joint through `ort` instead of `parakeet-rs`,
            // and prints the per-token evidence the crate discards.
            let Some(path) = args.get(2) else {
                eprintln!("Usage: transcrust --parakeet-direct /path/to/parakeet-model-dir [audio.wav]");
                std::process::exit(1);
            };
            let wav = args.get(3).map(std::path::Path::new);
            init_ort_default();
            let dir = std::path::Path::new(path);
            match model::parakeet_direct_graphs(dir) {
                Some(graphs) => {
                    println!("preprocessor: {}", graphs.preprocessor.display());
                    println!("encoder:      {}", graphs.encoder.display());
                    println!("joint:        {}", graphs.decoder_joint.display());
                }
                None => {
                    eprintln!("No direct-drive graph set in {path}.");
                    eprintln!("It needs nemo128.onnx alongside the encoder and decoder_joint:");
                    eprintln!("  curl -L -o {path}/nemo128.onnx \\");
                    eprintln!("    https://huggingface.co/istupakov/parakeet-tdt-0.6b-v3-onnx/resolve/main/nemo128.onnx");
                    std::process::exit(1);
                }
            }
            match parakeet_ort::run_model_smoke(dir, wav) {
                Ok(decoded) => {
                    let mut model = match parakeet_ort::LoadedParakeet::load(dir) {
                        Ok(model) => model,
                        Err(error) => {
                            eprintln!("{error}");
                            std::process::exit(1);
                        }
                    };
                    println!("text: {:?}", decoded.text);
                    println!("post-processed: {:?}", postprocess::fix_transcription(&decoded.text));
                    println!("{} tokens", decoded.tokens.len());
                    if let Some(first) = decoded.tokens.first() {
                        let last = decoded.tokens.last().unwrap_or(first);
                        println!(
                            "  speech spans {:.2}s–{:.2}s",
                            parakeet_ort::frame_to_seconds(first.frame),
                            parakeet_ort::frame_to_seconds(last.frame)
                        );
                    }
                    println!("  word confidences (min over sub-tokens):");
                    for (word, confidence) in decoded.word_confidences(model.vocabulary()) {
                        println!("    {confidence:>6.3}  {word}");
                    }
                    let _ = &mut model;
                }
                Err(error) => {
                    eprintln!("Parakeet direct drive failed: {error}");
                    std::process::exit(1);
                }
            }
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
        Some("--fix") => {
            let input = args[2..].join(" ");
            if input.is_empty() {
                eprintln!("Usage: transcrust --fix <text>");
                std::process::exit(1);
            }
            println!("{}", postprocess::fix_transcription(&input));
            return;
        }
        Some("--fix-long") => {
            // What `Granite — Long` would inject: the mode profile, then the
            // shared pipeline, in that order.
            let input = args[2..].join(" ");
            if input.is_empty() {
                eprintln!("Usage: transcrust --fix-long <text>");
                std::process::exit(1);
            }
            let shaped = mode::apply_profile(mode::Profile::Long, &input);
            println!("profile:  {shaped}");
            println!("injected: {}", postprocess::fix_transcription(&shaped));
            return;
        }
        Some("--keys") => {
            // Choosing a push-to-talk chord was guesswork: nothing reported what
            // the daemon actually receives, or that a chord only fires when every
            // modifier is already down when the trigger lands.
            let config = config::load();
            let _ = runtime();
            runtime().block_on(hotkey::probe_keys(config.hotkey.device.as_deref()));
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
            println!("transcrust — Observable push-to-talk voice input");
            println!();
            println!("Usage: transcrust [OPTION]");
            println!();
            println!("Options:");
            println!("  --probe-onnx <PATH>         Probe a single ONNX file across builder variants");
            println!("  --probe-suite <PATH...>     Probe multiple ONNX files across builder variants");
            println!("  --granite-smoke <DIR>       Run Granite frontend, ONNX, CTC, and tokenizer");
            println!("  --parakeet-direct <DIR> [WAV]  Drive Parakeet's graphs directly; print confidences");
            println!("  --smoke                     Run with terminal phase logging enabled");
            println!("  --long                      Run the daemon in toggle mode for long dictation");
            println!("  --toggle                    Tell a --long daemon to start or stop recording");
            println!("  --quit                      Ask a running transcrust instance to exit");
            println!("  --doctor                    Print phase-relevant environment info");
            println!("  --fix <TEXT>                Run the post-processing pipeline on TEXT and print it");
            println!("  --fix-long <TEXT>           Same, but through the \"— Long\" mode profile first");
            println!("  --wav <WAV...> [--mode M]   Transcribe files offline; write <name>.md beside each");
            println!("  --normalise [DIR] [--lists]  Clean a transcript on stdin with s1-mini");
            println!("  --bench <WAV...>            Time every installed engine on the same recordings");
            println!("  --record                    Record a clip to the corpus dir; Enter to stop");
            println!("  --download-model [MODEL]    Download a Parakeet model");
            println!();
            println!("Available models for --download-model:");
            println!("  parakeet-tdt-0.6b-int8      INT8 quantised, ~250 MB (default)");
            println!("  parakeet-tdt-0.6b-int4      INT4 quantised, ~409 MB (less RAM at inference)");
            println!("  parakeet-tdt-0.6b           FP32 full precision, ~1.4 GB");
            println!();
            println!("Models are saved to ~/.local/share/transcrust/models/");
            println!("  --keys                      Watch keyboard events; name the chord to bind");
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

    // Interface first, probes second.
    //
    // `run_probe_suite` fails if *any* builder variant fails, and a model with
    // external weights (`*.onnx_data`) can never pass the in-memory variants —
    // ORT resolves the sidecar relative to a file path it does not have. Exiting
    // on that denied the interface listing to precisely the models whose
    // interface is least guessable.
    for path in paths {
        println!("\n{}", path.display());
        match ort::session::Session::builder().and_then(|mut b| b.commit_from_file(path)) {
            Ok(session) => {
                for input in session.inputs() {
                    println!("  in   {input:?}");
                }
                for output in session.outputs() {
                    println!("  out  {output:?}");
                }
            }
            Err(error) => println!("  could not open for interface listing: {error}"),
        }
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

    // Modes, not models: `Granite — Long` is the same directory with a
    // different profile, so the switchable unit carries both.
    let installed = mode::discover_modes(config.model.path.as_deref());
    if installed.is_empty() {
        observer.error("startup", "No supported ASR model found");
        for path in model::explain_search_paths() {
            observer.phase("startup", &format!("searched: {path}"));
        }
        return;
    }
    for found in &installed {
        observer.phase(
            "startup",
            &format!("found {}: {}", found.label, found.model.path.display()),
        );
    }

    // Head of the list is the default; the tray can move to any other entry.
    let mut active_engine = 0usize;
    let model_path = installed[active_engine].model.path.clone();
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
    let (engine_request_tx, mut engine_request_rx) = tokio::sync::mpsc::unbounded_channel();
    let (active_engine_tx, active_engine_rx) = tokio::sync::watch::channel(active_engine);
    spawn(tray::run_tray(
        state.rx.clone(),
        observer.log_path().display().to_string(),
        installed.iter().map(|model| model.label.clone()).collect(),
        active_engine_rx,
        engine_request_tx,
    ));
    let (hotkey_tx_for_toggle, mut hotkey_rx) = hotkey::listen(&config.hotkey).await;
    let mut transcription = match build_service(&installed[active_engine].model, &config) {
        Ok(service) => service,
        Err(e) => {
            observer.error("startup", &e);
            return;
        }
    };
    let mut active_audio_rx: Option<std::sync::mpsc::Receiver<Vec<f32>>> = None;

    // Only a --long daemon listens for SIGUSR1. Registered before the loop so a
    // toggle that arrives during model load is queued rather than killing the
    // process — the default disposition for SIGUSR1 is terminate.
    // **Always** register, even outside long mode.
    //
    // The default disposition for SIGUSR1 is *terminate*, so a daemon that does
    // not handle it dies silently the first time someone runs
    // `transcrust --toggle` against it — measured, not theorised. Catching it
    // unconditionally turns a lost dictation session into a log line.
    let mut toggle_signal =
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::user_defined1()) {
            Ok(stream) => {
                if run_mode.long {
                    observer.phase(
                        "startup",
                        "long mode: recording starts and stops on `transcrust --toggle`",
                    );
                }
                Some(stream)
            }
            Err(error) => {
                observer.error("startup", &format!("failed to listen for SIGUSR1: {error}"));
                None
            }
        };


    loop {
        tokio::select! {
            Some(()) = async {
                match toggle_signal.as_mut() {
                    Some(stream) => stream.recv().await,
                    // No SIGUSR1 stream outside --long mode: park forever so
                    // this arm never fires and never busy-loops.
                    None => std::future::pending().await,
                }
            } => {
                if !run_mode.long {
                    observer.phase(
                        "toggle",
                        "ignored: daemon is in hold-to-talk mode; start it with --long",
                    );
                    observer.notify("Transcrust", "Not in --long mode; toggle ignored");
                    continue;
                }
                // The state machine already guards both transitions, so a
                // toggle is a translation rather than a new path.
                let event = match state.current() {
                    state::AppState::Idle => hotkey::HotkeyEvent::Pressed,
                    state::AppState::Recording => hotkey::HotkeyEvent::Released,
                    other => {
                        observer.phase(
                            "toggle",
                            &format!("ignored: busy ({other:?})"),
                        );
                        continue;
                    }
                };
                if hotkey_tx_for_toggle.send(event).await.is_err() {
                    observer.error("toggle", "hotkey channel closed");
                }
            }
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
                    hotkey::HotkeyEvent::CycleMode => {
                        // Same contract as the tray radio: only when idle, and
                        // the watch channel is what tells the tray where we
                        // actually landed.
                        if installed.len() < 2 {
                            observer.phase("engine.switch", "only one mode installed");
                        } else if state.current() != state::AppState::Idle {
                            observer.phase("engine.switch", "ignored: not idle");
                            observer.notify("Transcrust", "Busy — finish the current dictation first");
                        } else {
                            let next = (active_engine + 1) % installed.len();
                            let target = &installed[next];
                            // Two modes over one model share a service; no
                            // reload, no 527 MB round trip.
                            let same_model = installed[active_engine].model.path == target.model.path;
                            let outcome = if same_model {
                                Ok(transcription.clone())
                            } else {
                                build_service(&target.model, &config)
                            };
                            match outcome {
                                Ok(service) => {
                                    transcription = service;
                                    active_engine = next;
                                    let _ = active_engine_tx.send(active_engine);
                                    observer.phase("engine.switch", &format!("active: {}", target.label));
                                    observer.notify("Transcrust", &format!("Mode: {}", target.label));
                                }
                                Err(e) => {
                                    observer.error("engine.switch", &e);
                                    let _ = active_engine_tx.send(active_engine);
                                }
                            }
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
                            let profile = installed[active_engine].profile;
                            let mode_label = installed[active_engine].label.clone();
                            let engine_name = installed[active_engine].model.kind.display_name().to_string();
                            let corpus_enabled = config.observe.corpus;
                            let transcription = transcription.clone();
                            let observer = observer.clone();

                            spawn(async move {
                                run_transcription_pipeline(
                                    observer,
                                    transcription,
                                    audio_rx,
                                    sample_rate,
                                    output_cfg,
                                    profile,
                                    mode_label,
                                    engine_name,
                                    corpus_enabled,
                                    run_mode.long,
                                    state,
                                ).await;
                            });
                        }
                    }
                }
            }
            Some(requested) = engine_request_rx.recv() => {
                // Refuse mid-utterance: swapping the service under a running job
                // would strand the audio receiver the worker is draining.
                // The tray moves its radio optimistically, so every path that
                // declines the switch has to put it back or the menu will claim
                // an engine that was never loaded.
                if state.current() != state::AppState::Idle {
                    observer.phase("engine.switch", "ignored: not idle");
                    observer.notify("Transcrust", "Busy — finish the current dictation first");
                    let _ = active_engine_tx.send(active_engine);
                    continue;
                }
                let Some(target) = installed.get(requested) else {
                    observer.error("engine.switch", &format!("no model at index {requested}"));
                    let _ = active_engine_tx.send(active_engine);
                    continue;
                };
                if requested == active_engine {
                    continue;
                }
                observer.phase("engine.switch", &format!("switching to {}", target.label));
                let same_model = installed[active_engine].model.path == target.model.path;
                let outcome = if same_model {
                    Ok(transcription.clone())
                } else {
                    build_service(&target.model, &config)
                };
                match outcome {
                    Ok(service) => {
                        // The outgoing worker holds its model until its own idle
                        // timeout fires; nothing here forces it out early.
                        transcription = service;
                        active_engine = requested;
                        let _ = active_engine_tx.send(active_engine);
                        observer.phase("engine.switch", &format!("active: {}", target.label));
                        observer.notify("Transcrust", &format!("Engine: {}", target.label));
                    }
                    Err(e) => {
                        observer.error("engine.switch", &e);
                        // Put the radio back on the model that is really loaded.
                        let _ = active_engine_tx.send(active_engine);
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

fn build_service(
    model: &model::InstalledModel,
    config: &config::Config,
) -> Result<transcription::TranscriptionService, String> {
    transcription::TranscriptionService::new(
        model.path.to_string_lossy().into_owned(),
        model.kind,
        config.observe.idle_timeout_secs,
    )
}

/// Feed a long capture through the engine one window at a time.
///
/// The same windowing `--wav` uses, against the same seam, so the two paths
/// cannot drift: `wav::plan_windows` cuts at the quietest 20 ms frame near each
/// 60 s boundary, and each window goes to the engine as its own job. Windows
/// tile exactly — no sample dropped, none heard twice — which is pinned by
/// `wav.rs`'s own tests.
async fn transcribe_windowed(
    observer: &observe::Observer,
    transcription: &transcription::TranscriptionService,
    samples: &[f32],
    sample_rate: u32,
) -> Result<String, String> {
    let windows = wav::plan_windows(samples.len(), sample_rate, |from, to| {
        samples[from..to].iter().map(|s| s * s).sum::<f32>()
    });
    observer.phase(
        "transcription",
        &format!("{} window(s) to transcribe", windows.len()),
    );

    let mut parts: Vec<String> = Vec::new();
    for (index, window) in windows.iter().enumerate() {
        let (tx, rx) = std::sync::mpsc::channel();
        // One chunk then drop is what the live capture path looks like to the
        // worker once recording has stopped.
        tx.send(samples[window.start..window.end].to_vec())
            .map_err(|_| "engine receiver closed".to_string())?;
        drop(tx);

        let text = transcription
            .transcribe(observer.clone(), rx, sample_rate)
            .await?;
        observer.phase(
            "transcription",
            &format!("window {}/{} done", index + 1, windows.len()),
        );
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            parts.push(trimmed.to_string());
        }
    }

    Ok(parts.join(" "))
}

async fn run_transcription_pipeline(
    observer: observe::Observer,
    transcription: transcription::TranscriptionService,
    audio_rx: std::sync::mpsc::Receiver<Vec<f32>>,
    sample_rate: u32,
    output_cfg: config::OutputConfig,
    profile: mode::Profile,
    mode_label: String,
    engine_name: String,
    corpus_enabled: bool,
    long_form: bool,
    state: Arc<state::StateMachine>,
) {
    observer.phase("transcription", &format!("starting {} transcription", transcription.name()));

    // Tee the audio before the engine drains it. Recording has already been
    // stopped by the caller, so the channel is fully buffered and this does not
    // block; the engines collect the whole utterance anyway, so nothing about
    // their behaviour changes.
    let (audio_rx, banked) = if corpus_enabled {
        let mut samples: Vec<f32> = Vec::new();
        while let Ok(chunk) = audio_rx.recv() {
            samples.extend_from_slice(&chunk);
        }
        let (tx, rx) = std::sync::mpsc::channel();
        let _ = tx.send(samples.clone());
        drop(tx);
        (rx, Some(samples))
    } else {
        (audio_rx, None)
    };

    let started = std::time::Instant::now();

    // Long mode drains and windows; the default path is left exactly as it was.
    //
    // Two ceilings sit between a toggle-length dictation and a transcript, and
    // both are invisible until you cross them:
    //
    //  * the engine's own 45 s budget, which at Parakeet's ~0.2 RTF is spent by
    //    about 225 s of speech, and
    //  * Parakeet's encoder, which past roughly five minutes does not slow down
    //    but *throws* — `2501 by 7501`, a positional table meeting a longer
    //    sequence.
    //
    // `wav.rs` already solved this for files: cut at the quietest frame near
    // each 60 s boundary and feed the seam one window at a time. Reusing it here
    // means a two-minute dictation and a two-minute WAV take the same path.
    let result = if long_form {
        let mut samples: Vec<f32> = Vec::new();
        while let Ok(chunk) = audio_rx.recv() {
            samples.extend_from_slice(&chunk);
        }
        let seconds = samples.len() as f64 / sample_rate.max(1) as f64;
        // Scale the ceiling with the work rather than removing it: a hung engine
        // should still surrender. Generous, because the point is not to abort a
        // dictation the user cannot re-record.
        let budget = std::time::Duration::from_secs_f64((seconds * 2.0).max(45.0));
        observer.phase(
            "transcription",
            &format!("long mode: {seconds:.0}s captured, {}s budget", budget.as_secs()),
        );
        match tokio::time::timeout(
            budget,
            transcribe_windowed(&observer, &transcription, &samples, sample_rate),
        )
        .await
        {
            Ok(result) => result,
            Err(_) => {
                observer.error("transcription", &format!("timed out after {}s", budget.as_secs()));
                state.transition(state::AppState::Error);
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                state.transition(state::AppState::Idle);
                return;
            }
        }
    } else {
        // The ceiling is the engine's, not a global constant: VibeVoice decodes
        // at roughly real time and 45s would abort any dictation over a minute.
        let budget = transcription.timeout();
        match tokio::time::timeout(
            budget,
            transcription.transcribe(observer.clone(), audio_rx, sample_rate),
        )
        .await
        {
            Ok(result) => result,
            Err(_) => {
                observer.error("transcription", &format!("timed out after {}s", budget.as_secs()));
                state.transition(state::AppState::Error);
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                state.transition(state::AppState::Idle);
                return;
            }
        }
    };

    let transcribe_secs = started.elapsed().as_secs_f32();

    match result {
        Ok(text) if !text.is_empty() => {
            observer.sample("transcription.raw", &text);
            // The profile runs before the shared seam, per the post-processing
            // contract: anything engine- or mode-specific belongs here, not
            // inside fix_transcription.
            let shaped = mode::apply_profile(profile, &text);
            if shaped != text {
                observer.sample("transcription.profile", &shaped);
            }
            let fixed = postprocess::fix_transcription(&shaped);
            observer.sample("transcription.postprocess", &fixed);
            state.transition(state::AppState::Injecting);
            observer.phase("inject", "injecting transcript");

            match inject::inject_text(&fixed, &output_cfg).await {
                // A method that was asked for and failed is reported even when
                // another one carried the transcript, and notified as well as
                // logged — the whole failure mode here was being invisible while
                // the clipboard quietly absorbed it.
                Ok(warnings) => {
                    for warning in &warnings {
                        observer.error("inject", warning);
                    }
                    if !warnings.is_empty() {
                        observer.notify(
                            "Transcrust: not typed",
                            "Transcript is on the clipboard. See the log for why typing failed.",
                        );
                    }
                }
                Err(e) => {
                    observer.error("inject", &e);
                    state.transition(state::AppState::Error);
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    state.transition(state::AppState::Idle);
                    return;
                }
            }

            if let Some(samples) = banked {
                bank(&observer, &samples, sample_rate, &mode_label, &engine_name,
                     &text, &fixed, transcribe_secs);
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

/// Run every installed engine over the same recordings and print what each one
/// cost.
///
/// This goes through `TranscriptionService::transcribe`, the same call the
/// hotkey path makes, so the numbers include resampling and the shared
/// post-processing — not just the ONNX graph.
///
/// Load is timed separately because it is paid once per engine, not once per
/// utterance. The workers load lazily on their first job, so each engine gets a
/// throwaway half-second of silence first; that call is the load measurement,
/// and every clip after it is warm.
async fn run_bench(paths: &[std::path::PathBuf]) {
    const WARMUP_RATE: u32 = 48_000;

    let config = config::load();
    let observer = observe::Observer::new(config.observe.sample_chars, false, false)
        .expect("Failed to initialize bench observer");
    // Modes, not models: `Granite — Long` is the same directory with a
    // different profile, so the switchable unit carries both.
    let installed = mode::discover_modes(config.model.path.as_deref());
    if installed.is_empty() {
        eprintln!("No supported ASR model found");
        std::process::exit(1);
    }

    let mut clips = Vec::new();
    for path in paths {
        match audio::read_wav_mono(path) {
            Ok((samples, rate)) => clips.push((path.clone(), samples, rate)),
            Err(error) => {
                eprintln!("{error}");
                std::process::exit(1);
            }
        }
    }

    async fn feed(
        service: &transcription::TranscriptionService,
        observer: &observe::Observer,
        samples: &[f32],
        rate: u32,
    ) -> (Result<String, String>, std::time::Duration) {
        let (tx, rx) = std::sync::mpsc::channel();
        // The live path streams chunks from the capture thread and the worker
        // drains until the sender drops; one chunk then drop reproduces that.
        tx.send(samples.to_vec()).expect("bench receiver is alive");
        drop(tx);
        let started = std::time::Instant::now();
        let result = service.transcribe(observer.clone(), rx, rate).await;
        (result, started.elapsed())
    }

    println!(
        "{:<46} {:>8} {:>10} {:>7}",
        "engine / clip", "audio", "wall", "RTF"
    );
    for found in &installed {
        let service = match build_service(&found.model, &config) {
            Ok(service) => service,
            Err(error) => {
                println!("{:<46} {error}", found.label);
                continue;
            }
        };

        let silence = vec![0.0f32; WARMUP_RATE as usize / 2];
        let (_, load) = feed(&service, &observer, &silence, WARMUP_RATE).await;
        println!(
            "{:<46} {:>8} {:>9.2}s {:>7}",
            format!("{} / cold load", found.label),
            "-",
            load.as_secs_f64(),
            "-"
        );

        for (path, samples, rate) in &clips {
            let (result, elapsed) = feed(&service, &observer, samples, *rate).await;
            let seconds = samples.len() as f64 / *rate as f64;
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            match result {
                Ok(text) => {
                    println!(
                        "{:<46} {:>7.2}s {:>9.2}s {:>7.2}",
                        format!("{} / {name}", found.label),
                        seconds,
                        elapsed.as_secs_f64(),
                        elapsed.as_secs_f64() / seconds
                    );
                    println!("      {:?}", postprocess::fix_transcription(&text));
                }
                Err(error) => println!("{:<46} {error}", format!("{} / {name}", found.label)),
            }
        }
    }

    bench_direct_driver(&installed, &clips);
}

/// The fourth row: Parakeet driven through `ort` directly instead of through
/// the `parakeet-rs` crate.
///
/// `parakeet_ort.rs` has existed, passing its own tests, with nothing calling it
/// but `--parakeet-direct`. Its kill criterion — *more than 15% slower than the
/// crate and you profile before proceeding* — was written and never evaluated,
/// which is why it sits here rather than in a doc: the comparison has to be on
/// the same clips, in the same run, next to the number it is judged against.
///
/// It bypasses `TranscriptionService` on purpose. The whole question is what the
/// crate boundary costs, so routing this through the seam the crate sits behind
/// would measure nothing.
fn bench_direct_driver(installed: &[mode::Mode], clips: &[(std::path::PathBuf, Vec<f32>, u32)]) {
    let Some(parakeet) = installed
        .iter()
        .find(|mode| mode.model.kind == model::ModelKind::Parakeet)
    else {
        return;
    };
    let dir = parakeet.model.path.as_path();
    if model::parakeet_direct_graphs(dir).is_none() {
        println!(
            "{:<46} needs nemo128.onnx alongside the encoder and joint",
            "Parakeet direct (ort) / unavailable"
        );
        return;
    }

    let label = "Parakeet direct (ort)";
    let started = std::time::Instant::now();
    let mut model = match parakeet_ort::LoadedParakeet::load(dir) {
        Ok(model) => model,
        Err(error) => {
            println!("{:<46} {error}", format!("{label} / load"));
            return;
        }
    };
    println!(
        "{:<46} {:>8} {:>9.2}s {:>7}",
        format!("{label} / cold load"),
        "-",
        started.elapsed().as_secs_f64(),
        "-"
    );

    for (path, samples, rate) in clips {
        // The crate resamples internally; this driver does not, so match what
        // the live path would hand it rather than measuring a resample twice.
        let audio = audio::resample_to_16k(samples, *rate);
        let seconds = samples.len() as f64 / *rate as f64;
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();

        let started = std::time::Instant::now();
        match model.transcribe(&audio) {
            Ok(decoded) => {
                let elapsed = started.elapsed();
                println!(
                    "{:<46} {:>7.2}s {:>9.2}s {:>7.2}",
                    format!("{label} / {name}"),
                    seconds,
                    elapsed.as_secs_f64(),
                    elapsed.as_secs_f64() / seconds
                );
                println!(
                    "      {:?}",
                    postprocess::fix_transcription(&decoded.text)
                );
                // The signal the crate throws away, and the reason this driver
                // exists at all. Anything below the gate is what D.0 would
                // surface to the user instead of making them read the line.
                let vocab = model.vocabulary();
                let low: Vec<String> = decoded
                    .word_confidences(vocab)
                    .into_iter()
                    .filter(|(_, confidence)| *confidence < 0.75)
                    .map(|(word, confidence)| format!("{word} {confidence:.2}"))
                    .collect();
                if low.is_empty() {
                    println!("      confidence: every word above 0.75");
                } else {
                    println!("      below 0.75: {}", low.join(", "));
                }
            }
            Err(error) => println!("{:<46} {error}", format!("{label} / {name}")),
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn bank(
    observer: &observe::Observer,
    samples: &[f32],
    sample_rate: u32,
    mode_label: &str,
    engine_name: &str,
    raw: &str,
    injected: &str,
    transcribe_secs: f32,
) {
    let entry = corpus::Entry {
        recorded: corpus::stamp(),
        mode: mode_label.to_string(),
        engine: engine_name.to_string(),
        device_sample_rate: sample_rate,
        duration_secs: corpus::duration_of(samples.len(), sample_rate).as_secs_f32(),
        raw: raw.to_string(),
        injected: injected.to_string(),
        transcribe_secs,
        reference: None,
    };
    match corpus::save(samples, sample_rate, &entry) {
        Ok(path) => observer.phase(
            "corpus",
            &format!(
                "banked {} ({:.1}s) — fill `reference` in the .json if this one came out wrong",
                path.display(),
                entry.duration_secs
            ),
        ),
        Err(error) => observer.error("corpus", &error),
    }
}

/// Record one clip into the corpus directory, with a per-second status line.
/// Enter stops it. No transcription, no model load.
async fn run_record() {
    let capture = match audio::AudioCapture::new(config::load().audio.device.as_deref()) {
        Ok(capture) => capture,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };
    let sample_rate = capture.sample_rate();
    println!("Recording at {sample_rate} Hz mono (channel 0 of the device).");
    println!("Press Enter to stop.");

    let rx = capture.start_recording();
    let started = std::time::Instant::now();

    // stdin read has to be off the async runtime; a blocking task is the
    // supported way to park a thread on it.
    let stop = tokio::task::spawn_blocking(|| {
        let mut line = String::new();
        let _ = std::io::stdin().read_line(&mut line);
    });
    tokio::pin!(stop);

    let mut ticker = tokio::time::interval(std::time::Duration::from_secs(1));
    loop {
        tokio::select! {
            _ = &mut stop => break,
            _ = ticker.tick() => {
                print!("\r  ● recording  {}   ", corpus::clock(started.elapsed()));
                use std::io::Write;
                let _ = std::io::stdout().flush();
            }
        }
    }
    capture.stop_recording();

    let mut samples = Vec::new();
    while let Ok(chunk) = rx.recv() {
        samples.extend_from_slice(&chunk);
    }
    let duration = corpus::duration_of(samples.len(), sample_rate);

    if let Err(error) = std::fs::create_dir_all(corpus::dir()) {
        eprintln!("\nfailed to create corpus dir: {error}");
        std::process::exit(1);
    }
    let path = corpus::dir().join(format!("{}.wav", corpus::stamp()));
    match corpus::write_wav(&path, &samples, sample_rate) {
        Ok(()) => println!(
            "\r  ✓ saved  {}  ({}, {} samples)      ",
            path.display(),
            corpus::clock(duration),
            samples.len()
        ),
        Err(error) => {
            eprintln!("\n{error}");
            std::process::exit(1);
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
    let resolved = model::find_model_path(config.model.path.as_deref());
    match &resolved {
        Some(path) => println!("Resolved model: {}", path.display()),
        None => println!("Resolved model: <missing>"),
    }
    match resolved.as_deref().and_then(model::model_kind) {
        Some(model::ModelKind::Parakeet) => println!("Engine: Parakeet TDT"),
        Some(model::ModelKind::Granite) => println!("Engine: Granite Speech 5 TurboCTC"),
        None => println!("Engine: <none>"),
    }
    println!("Search paths:");
    for path in model::explain_search_paths() {
        println!("  {path}");
    }
    // Same list, same order, that the tray's Engine submenu offers. Modes, not
    // models: a `— Long` entry is the same directory with a repair profile.
    let modes = mode::discover_modes(config.model.path.as_deref());
    println!("Modes ({}):", modes.len());
    for (index, found) in modes.iter().enumerate() {
        let marker = if index == 0 { "*" } else { " " };
        let profile = match found.profile {
            mode::Profile::Raw => "raw",
            mode::Profile::Long => "long-form repair",
        };
        println!(
            "  {marker} {} [{profile}] — {}",
            found.label,
            found.model.path.display()
        );
    }
    if modes.len() < 2 {
        println!("  (tray Engine switcher appears once two or more are installed)");
    }
    match resolved.as_deref() {
        Some(path) if model::has_parakeet_direct(path) => {
            println!("Parakeet direct drive: available (nemo128 present)")
        }
        Some(_) => println!(
            "Parakeet direct drive: unavailable (needs nemo128.onnx; see --parakeet-direct)"
        ),
        None => {}
    }
    // The push-to-talk binding, and whether it can leak into your document.
    // Resolved the same way the listener resolves it, so this reports the
    // binding actually in force rather than the fields it was written in.
    let binding = hotkey::resolve_binding(&config.hotkey);
    let (bound_modifiers, bound_key) = match &binding {
        Ok(pair) => pair.clone(),
        Err(error) => {
            println!("Hotkey: INVALID — {error}");
            println!("  transcrust will refuse to start until this is fixed.");
            (Vec::new(), String::new())
        }
    };
    if binding.is_ok() {
        let mut parts = bound_modifiers.clone();
        parts.push(bound_key.clone());
        println!("Hotkey: {} (hold to talk)", parts.join("+"));
        if config.hotkey.grab {
            println!("  Grab: on — keyboard is held exclusively for the duration of the press");
        }
    }
    if binding.is_ok() && !hotkey::is_silent_key(&bound_key) {
        println!("  ⚠ trigger \"{bound_key}\" produces a character.");
        if config.hotkey.grab {
            println!("    grab is on, so the auto-repeat is suppressed — but the first press");
            println!("    reaches the focused window before the grab takes effect, so expect");
            println!("    one stray keystroke per hold.");
        } else {
            println!("    transcrust reads evdev passively and does not grab the keyboard, so");
            println!("    every press and auto-repeat reaches the focused window for the whole");
            println!("    hold. Set hotkey.grab = true to stop the repeat, or pick a silent key.");
        }
        println!("    Silent triggers: any modifier, F13-F20, Pause, ScrollLock, Print.");
    }
    match (config.hotkey.mode_key.as_deref(), config.hotkey.mode_modifiers.as_slice()) {
        (Some(key), mods) if !mods.is_empty() => {
            println!("Mode toggle: {}+{key}", mods.join("+"))
        }
        (Some(key), _) => println!("Mode toggle: {key}"),
        (None, _) => println!("Mode toggle: <unbound> (set hotkey.mode_key to enable)"),
    }
    println!("Preferred int8 model dir: {}", model::preferred_int8_model_dir().display());
    println!("Required Parakeet files:");
    for file in model::required_int8_files() {
        println!("  {file}");
    }
    println!("Required Granite files:");
    for file in model::required_granite_files() {
        println!("  {file}");
    }
    // The correction/vocabulary leg is engine-agnostic: both engines return raw
    // text and main.rs runs postprocess::fix_transcription on it exactly once.
    println!("Post-processing: shared by all engines (postprocess::fix_transcription)");
    let dict_path = dictionary::dictionary_path();
    let dict_entries = std::fs::read_to_string(&dict_path)
        .map(|c| {
            c.lines()
                .map(str::trim)
                .filter(|l| !l.is_empty() && !l.starts_with('#'))
                .count()
        })
        .ok();
    match dict_entries {
        Some(n) => println!("Phonetic dictionary: {} ({n} entries)", dict_path.display()),
        None => println!("Phonetic dictionary: {} (absent — pass-through)", dict_path.display()),
    }
    // Dragon mechanism 7: fail loudly on bad input instead of quietly
    // transcribing mush. The resampler defect A.1 fixed went unnoticed for
    // months because nothing was watching this path, so the response is
    // measured here rather than asserted.
    println!("Audio path:");
    match audio::default_input_summary(config.audio.device.as_deref()) {
        Ok((name, rate, channels)) => {
            println!("  Device: {name} @ {rate} Hz, {channels}ch");
            if channels > 1 {
                println!("  Downmix: channel 0 only (capture does not sum channels)");
            }
            if rate == 16_000 {
                println!("  Resample: none needed — device is already at 16 kHz");
            } else {
                println!("  Resample: {rate} -> 16000, band-limited polyphase sinc");
                let response = audio::resampler_response(rate, 16_000);
                if response.is_empty() {
                    println!("    (no probe frequency aliases at this rate)");
                }
                for (probe, db, folds) in response {
                    let verdict = if db <= -40.0 { "ok" } else { "LEAKS" };
                    println!(
                        "    {:>6.0} Hz  {:>7.1} dB  would fold onto {:>5.0} Hz  {verdict}",
                        probe, db, folds
                    );
                }
            }
        }
        Err(error) => println!("  Device: unavailable ({error})"),
    }
    println!("  Clipping: not measurable here — record a clip with --record and check levels");

    println!("Quit pid file: {}", control::pid_file_path().display());
    let mut typing_available = false;
    for cmd in ["wtype", "dotool", "notify-send"] {
        let found = std::process::Command::new("sh")
            .arg("-lc")
            .arg(format!("command -v {cmd} >/dev/null"))
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if found && (cmd == "wtype" || cmd == "dotool") {
            typing_available = true;
        }
        println!("Command {cmd}: {}", if found { "yes" } else { "no" });
    }
    // The failure this catches is silent by construction: clipboard output is an
    // in-process library call that essentially always succeeds, so with no typing
    // tool the transcript still lands on the clipboard and nothing is typed.
    if config.output.wtype && !typing_available {
        println!("  ⚠ output.wtype is on but neither wtype nor dotool is on PATH.");
        println!("    Transcripts will reach the clipboard and never be typed.");
        println!("    Both live in the nix devShell: run via 'nix develop -c ...',");
        println!("    or use the wrapped install rather than ./target/release/transcrust.");
    }
}
