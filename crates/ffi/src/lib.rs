//! C ownership boundary for native mobile adapters. UI and delivery stay outside.
use serde_json::{json, Value};
use std::ffi::{c_char, CStr, CString};
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

pub struct TcEngine {
    state: Mutex<State>,
    cancel: Arc<AtomicBool>,
    cancel_requested: AtomicBool,
}
struct State {
    vocabulary: PathBuf,
    #[cfg(feature = "decision")]
    decision: Option<transcrust_decision::Decision>,
    #[cfg(feature = "decision")]
    runtime: tokio::runtime::Runtime,
    #[cfg(feature = "asr")]
    asr: Option<Asr>,
}
#[cfg(feature = "asr")]
enum Asr {
    Parakeet(Box<transcrust_core::parakeet_ort::LoadedParakeet>),
    Nemotron(Box<transcrust_core::nemotron::Nemotron>),
}

fn required<'a>(v: &'a Value, key: &str) -> Result<&'a str, String> {
    v[key]
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| format!("{key} required"))
}

/// Blocking; call from an application worker. error receives allocated JSON or
/// null. No config/model files are downloaded. Handles are independent.
#[no_mangle]
pub unsafe extern "C" fn tc_create(
    config: *const c_char,
    error: *mut *mut c_char,
) -> *mut TcEngine {
    if !error.is_null() {
        *error = std::ptr::null_mut();
    }
    let result = (|| -> Result<TcEngine, String> {
        let v: Value = serde_json::from_str(text(config)?).map_err(|e| e.to_string())?;
        let vocabulary = PathBuf::from(required(&v, "vocabulary_path")?);
        #[cfg(not(feature = "decision"))]
        if !v["decision_backend"].is_null() {
            return Err("library built without decision feature".into());
        }
        #[cfg(not(feature = "asr"))]
        if !v["asr"].is_null() {
            return Err("library built without asr feature".into());
        }
        #[cfg(feature = "decision")]
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .map_err(|e| e.to_string())?;
        #[cfg(feature = "decision")]
        let decision = if v["decision_backend"].is_null() {
            None
        } else {
            let backend = match required(&v, "decision_backend")? {
                "ultra" => transcrust_decision::Backend::Ultra,
                "d1" => transcrust_decision::Backend::D1,
                _ => return Err("decision_backend must be ultra or d1".into()),
            };
            Some(runtime.block_on(transcrust_decision::Decision::start_at(
                &transcrust_core::SilentEvents,
                backend,
                PathBuf::from(required(&v, "decision_model_path")?),
                vocabulary.clone(),
            )))
        };
        #[cfg(feature = "asr")]
        let asr = if v["asr"].is_null() {
            None
        } else {
            transcrust_core::init_ort_default();
            let dir = std::path::Path::new(required(&v, "asr_model_path")?);
            Some(match required(&v, "asr")? {
                "parakeet" => Asr::Parakeet(Box::new(
                    transcrust_core::parakeet_ort::LoadedParakeet::load(dir)?,
                )),
                "nemotron" => Asr::Nemotron(Box::new(transcrust_core::nemotron::Nemotron::load(
                    dir,
                    transcrust_core::nemotron::Tuning::for_export(dir),
                )?)),
                _ => return Err("asr must be parakeet or nemotron".into()),
            })
        };
        Ok(TcEngine {
            state: Mutex::new(State {
                vocabulary,
                #[cfg(feature = "decision")]
                decision,
                #[cfg(feature = "decision")]
                runtime,
                #[cfg(feature = "asr")]
                asr,
            }),
            cancel: Arc::new(AtomicBool::new(false)),
            cancel_requested: AtomicBool::new(false),
        })
    })();
    match result {
        Ok(engine) => Box::into_raw(Box::new(engine)),
        Err(e) => {
            if !error.is_null() {
                *error = reply(json!({"error":e}));
            }
            std::ptr::null_mut()
        }
    }
}

impl State {
    fn process(&mut self, raw: &str, cancel: Arc<AtomicBool>, cancel_requested: &AtomicBool) -> Value {
        let cleaned = transcrust_core::postprocess::fix_transcription(raw);
        let mut out = json!({"raw":raw,"cleaned":cleaned,"text":cleaned});
        #[cfg(feature = "decision")]
        if let Some(model) = &self.decision {
            let result = self.runtime.block_on(model.adjudicate_with_cancel(
                &cleaned,
                &transcrust_core::SilentEvents,
                cancel.clone(),
            ));
            out["text"] = json!(result.text);
            out["audit"] = result.audit;
        }
        // An inference deadline also stops the worker. Only an explicit native
        // cancellation discards the take; deadlines retain text and rule edits.
        out["cancelled"] = json!(cancel_requested.load(Ordering::Relaxed));
        out
    }
}

/// Process text or explicitly accepted pairs. Never delivers to a field.
#[no_mangle]
pub unsafe extern "C" fn tc_process(engine: *mut TcEngine, request: *const c_char) -> *mut c_char {
    let result = (|| -> Result<Value, String> {
        let engine = engine.as_ref().ok_or("null engine")?;
        let value: Value = serde_json::from_str(text(request)?).map_err(|e| e.to_string())?;
        let mut state = engine
            .state
            .try_lock()
            .map_err(|_| "engine busy or poisoned")?;
        engine.cancel.store(false, Ordering::Relaxed);
        engine.cancel_requested.store(false, Ordering::Relaxed);
        match value["op"].as_str() {
            Some("learn") => {
                let pairs: Vec<transcrust_core::correction::Pair> =
                    serde_json::from_value(value["pairs"].clone()).map_err(|e| e.to_string())?;
                transcrust_core::vocabulary::save_pairs(&state.vocabulary, &pairs)?;
                Ok(json!({"saved":pairs.len()}))
            }
            Some("process") => Ok(state.process(
                required(&value, "text")?,
                engine.cancel.clone(),
                &engine.cancel_requested,
            )),
            _ => Err("op must be process or learn".into()),
        }
    })();
    match result {
        Ok(v) => reply(v),
        Err(e) => reply(json!({"error":e})),
    }
}

/// Mono PCM; blocking. Does ASR, deterministic cleanup and optional decisions.
/// The caller owns the input until return. Native ASR itself is not preempted.
#[no_mangle]
pub unsafe extern "C" fn tc_transcribe(
    engine: *mut TcEngine,
    pcm: *const f32,
    samples: usize,
    rate: u32,
) -> *mut c_char {
    let result = (|| -> Result<Value, String> {
        let engine = engine.as_ref().ok_or("null engine")?;
        if pcm.is_null()
            || samples > isize::MAX as usize / std::mem::size_of::<f32>()
            || !(8_000..=192_000).contains(&rate)
        {
            return Err("invalid PCM/rate".into());
        }
        let mut state = engine
            .state
            .try_lock()
            .map_err(|_| "engine busy or poisoned")?;
        engine.cancel.store(false, Ordering::Relaxed);
        engine.cancel_requested.store(false, Ordering::Relaxed);
        #[cfg(feature = "asr")]
        {
            let audio = std::slice::from_raw_parts(pcm, samples);
            if audio.iter().any(|v| !v.is_finite()) {
                return Err("non-finite PCM".into());
            }
            let audio = transcrust_core::audio::resample_to_16k(audio, rate);
            let raw = match state.asr.as_mut().ok_or("no ASR configured")? {
                Asr::Parakeet(model) => model.transcribe(&audio)?.text,
                Asr::Nemotron(model) => {
                    let mut front = model.streaming_frontend()?;
                    let mut stream = model.stream_begin();
                    if let Some(mel) = front.push(&audio)? {
                        model.stream_push(&mut stream, &mel)?;
                    }
                    if let Some(mel) = front.finish()? {
                        model.stream_push(&mut stream, &mel)?;
                    }
                    model.stream_finish(&mut stream)?.0
                }
            };
            Ok(state.process(&raw, engine.cancel.clone(), &engine.cancel_requested))
        }
        #[cfg(not(feature = "asr"))]
        {
            let _ = &mut state;
            Err("library built without asr feature".into())
        }
    })();
    match result {
        Ok(v) => reply(v),
        Err(e) => reply(json!({"error":e})),
    }
}

/// Concurrent cancel is allowed. Discard cancelled results in the native UI.
#[no_mangle]
pub unsafe extern "C" fn tc_cancel(engine: *mut TcEngine) {
    if let Some(engine) = engine.as_ref() {
        engine.cancel_requested.store(true, Ordering::Relaxed);
        engine.cancel.store(true, Ordering::Relaxed);
    }
}

/// Only after every call (including background work/cancellation) has returned.
#[no_mangle]
pub unsafe extern "C" fn tc_destroy(engine: *mut TcEngine) {
    if !engine.is_null() {
        drop(Box::from_raw(engine));
    }
}

fn reply(value: Value) -> *mut c_char {
    CString::new(value.to_string())
        .expect("JSON escapes NUL")
        .into_raw()
}
unsafe fn text<'a>(input: *const c_char) -> Result<&'a str, String> {
    if input.is_null() {
        return Err("null input".into());
    }
    CStr::from_ptr(input).to_str().map_err(|e| e.to_string())
}

/// Input is a JSON operation. Returned JSON is owned by the caller until tc_free.
/// No global vocabulary path is changed by these pure operations.
#[no_mangle]
pub unsafe extern "C" fn tc_text(request: *const c_char) -> *mut c_char {
    let result = (|| -> Result<Value, String> {
        let value: Value = serde_json::from_str(text(request)?).map_err(|e| e.to_string())?;
        match value["op"].as_str() {
            Some("clean") => Ok(
                json!({"text": transcrust_core::postprocess::fix_transcription(value["text"].as_str().ok_or("text required")?)}),
            ),
            Some("proposals") => Ok(
                json!({"pairs": transcrust_core::correction::proposals(value["before"].as_str().ok_or("before required")?, value["after"].as_str().ok_or("after required")?)}),
            ),
            // Correct-last review on mobile, parity with the desktop window: every change,
            // whether it can be learned, where Learn would file it, and whether the
            // corrected text is already an entry ("Matches ## Bluey").
            Some("review") => {
                let before = value["before"].as_str().ok_or("before required")?;
                let after = value["after"].as_str().ok_or("after required")?;
                let known = value["vocabulary_path"]
                    .as_str()
                    .map(|p| transcrust_core::vocabulary::canonicals(std::path::Path::new(p)))
                    .unwrap_or_default();
                let changes: Vec<Value> = transcrust_core::correction::changes(before, after)
                    .into_iter()
                    .map(|change| {
                        let pair = change.pair();
                        json!({
                            "before": change.before,
                            "after": change.after,
                            "heard": pair.heard,
                            "canonical": pair.canonical,
                            "learnable": transcrust_core::correction::learnable(&pair),
                            "field": if transcrust_core::correction::case_promotion(&pair) {
                                "capitalization"
                            } else {
                                transcrust_core::vocabulary::alias_field(&pair.heard)
                            },
                            "entry": known.iter().any(|k| *k == pair.canonical),
                        })
                    })
                    .collect();
                Ok(json!({ "changes": changes }))
            }
            _ => Err("op must be clean, proposals or review".into()),
        }
    })();
    match result {
        Ok(v) => reply(v),
        Err(e) => reply(json!({"error":e})),
    }
}

/// Free exactly once, using only a pointer returned by this library.
#[no_mangle]
pub unsafe extern "C" fn tc_free(value: *mut c_char) {
    if !value.is_null() {
        drop(CString::from_raw(value));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn worker_stop_preserves_text_unless_native_caller_cancelled() {
        unsafe {
            let config = CString::new(r#"{"vocabulary_path":"/unused/test-vocab.md"}"#).unwrap();
            let mut error = std::ptr::null_mut();
            let handle = tc_create(config.as_ptr(), &mut error);
            assert!(error.is_null());
            let engine = handle.as_ref().unwrap();
            // A deadline stops inference, while its fallback text stays deliverable.
            engine.cancel.store(true, Ordering::Relaxed);
            let response = engine.state.lock().unwrap().process(
                "My dog Gambol is here.",
                engine.cancel.clone(),
                &engine.cancel_requested,
            );
            assert_eq!(response["text"], "My dog Gambol is here.");
            assert_eq!(response["cancelled"], false);
            tc_cancel(handle);
            let response = engine.state.lock().unwrap().process(
                "Discard this take.",
                engine.cancel.clone(),
                &engine.cancel_requested,
            );
            assert_eq!(response["cancelled"], true);
            let request = CString::new(r#"{"op":"process","text":"The next take."}"#).unwrap();
            assert_eq!(take(tc_process(handle, request.as_ptr()))["cancelled"], false);
            tc_destroy(handle);
        }
    }

    unsafe fn take(p: *mut c_char) -> Value {
        assert!(!p.is_null());
        let v = serde_json::from_str(CStr::from_ptr(p).to_str().unwrap()).unwrap();
        tc_free(p);
        v
    }
    #[test]
    fn c_boundary_errors_ownership_and_persistent_learning() {
        unsafe {
            assert!(take(tc_text(std::ptr::null()))["error"].is_string());
            let request = CString::new(r#"{"op":"clean","text":"Hello café."}"#).unwrap();
            assert_eq!(take(tc_text(request.as_ptr()))["text"], "Hello café.");
            let dir = std::env::temp_dir().join(format!("transcrust-ffi-{}", std::process::id()));
            std::fs::create_dir_all(&dir).unwrap();
            let vocab = dir.join("vocab.md");
            std::fs::write(&vocab,"# Vocabulary\n\n## Gambol\ndescription: pet dog\nordinary: a gambling verb\n\n## Other\ndescription: preserved\n").unwrap();
            let config = CString::new(json!({"vocabulary_path":vocab}).to_string()).unwrap();
            let mut error = std::ptr::null_mut();
            let handle = tc_create(config.as_ptr(), &mut error);
            assert!(error.is_null());
            assert!(!handle.is_null());
            let review = CString::new(json!({"op":"review","vocabulary_path":vocab,
                "before":"My dog Gamble is here. I like Louis.","after":"My dog Gambol is here. I like Bluey!"}).to_string()).unwrap();
            let changes = take(tc_text(review.as_ptr()))["changes"].clone();
            assert_eq!(changes[0]["heard"], "Gamble");
            assert_eq!(changes[0]["canonical"], "Gambol");
            assert_eq!((changes[0]["entry"].as_bool(), changes[0]["learnable"].as_bool()), (Some(true), Some(true)));
            assert_eq!(changes[0]["field"], "homophones");
            // not an entry in this vocabulary
            assert_eq!(changes[1]["entry"], false);
            // a punctuation-only edit is listed but cannot be learned
            let punct = CString::new(r#"{"op":"review","before":"Hello there.","after":"Hello there!"}"#).unwrap();
            let punct = take(tc_text(punct.as_ptr()))["changes"].clone();
            assert_eq!(punct.as_array().unwrap().len(), 1);
            assert_eq!(punct[0]["learnable"], false);
            let proposals=CString::new(r#"{"op":"proposals","before":"My dog Gamble is here.","after":"My dog Gambol is here."}"#).unwrap();
            let pairs = take(tc_text(proposals.as_ptr()))["pairs"].clone();
            let learn = CString::new(json!({"op":"learn","pairs":pairs}).to_string()).unwrap();
            assert_eq!(take(tc_process(handle, learn.as_ptr()))["saved"], 1);
            let file = std::fs::read_to_string(&vocab).unwrap();
            assert!(file.contains("description: pet dog"));
            assert!(file.contains("ordinary: a gambling verb"));
            assert!(file.contains("description: preserved"));
            assert!(file.contains("Gamble"));
            // Separate handles carry distinct vocabulary paths, with no global reconfiguration.
            let other = dir.join("other.md");
            let config = CString::new(json!({"vocabulary_path":other}).to_string()).unwrap();
            let second = tc_create(config.as_ptr(), &mut error);
            assert!(!second.is_null());
            let check = take(tc_process(second, learn.as_ptr()));
            assert_eq!(check["saved"], 1);
            assert!(std::fs::read_to_string(other).unwrap().contains("Gambol"));
            tc_destroy(second);
            tc_destroy(handle);
            tc_free(std::ptr::null_mut());
            std::fs::remove_dir_all(dir).unwrap();
        }
    }
}
