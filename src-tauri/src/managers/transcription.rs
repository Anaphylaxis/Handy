use anyhow::Result;
use std::sync::Mutex;
use std::thread;
use tauri::{App, Manager};
use whisper_rs::install_whisper_log_trampoline;
use whisper_rs::{
    FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters, WhisperState,
};

pub struct TranscriptionManager {
    state: Mutex<WhisperState>,
}

impl TranscriptionManager {
    pub fn new(app: &App) -> Result<Self> {
        let whisper_path = app.path().resolve(
            "resources/models/ggml-small.bin",
            tauri::path::BaseDirectory::Resource,
        )?;
        let path = whisper_path
            .to_str()
            .expect("Path contains invalid UTF-8 Chars");

        install_whisper_log_trampoline();
        // Load the model
        let context = WhisperContext::new_with_params(path, WhisperContextParameters::default())
            .map_err(|e| anyhow::anyhow!("Failed to load whisper model: {}", e))?;

        // Create state
        let state = context.create_state().expect("failed to create state");

        Ok(Self {
            state: Mutex::new(state),
        })
    }

    pub fn transcribe(&self, audio: Vec<f32>) -> Result<String> {
        let st = std::time::Instant::now();

        let mut result = String::new();
        println!("Audio vector length: {}", audio.len());

        if audio.len() == 0 {
            println!("Empty audio vector");
            // TODO error
            return Ok(result);
        }

        let num_threads = thread::available_parallelism()
            .map(|count| count.get() as i32)
            .unwrap_or(1);

        let mut state = self.state.lock().unwrap();
        // Initialize parameters
        let mut params = FullParams::new(SamplingStrategy::default());
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_suppress_blank(true);
        params.set_suppress_non_speech_tokens(true);

        state
            .full(params, &audio)
            .expect("failed to convert samples");

        let num_segments = state
            .full_n_segments()
            .expect("failed to get number of segments");

        for i in 0..num_segments {
            let segment = state
                .full_get_segment_text(i)
                .expect("failed to get segment");
            result.push_str(&segment);
        }

        let et = std::time::Instant::now();
        println!("\n\ntook {}ms", (et - st).as_millis());

        Ok(result.trim().to_string())
    }
}
