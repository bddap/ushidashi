mod api;
mod audio;
mod button;
mod config;
mod prompt;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::api::OpenAIApiClient;
use crate::audio::to_wav;
use crate::button::is_spacebar_pressed;

#[tokio::main]
async fn main() {
    match run().await {
        Ok(()) => (),
        Err(e) => {
            eprintln!("Error: {:#?}", e);
        }
    }
}

async fn run() -> anyhow::Result<()> {
    let secrets = config::Secrets::load()?;

    let client = OpenAIApiClient::new(&secrets.openai_api_key);

    crate::button::start();

    loop {
        while !is_spacebar_pressed() {
            std::thread::sleep(Duration::from_millis(16));
        }
        let wav = record(|| !is_spacebar_pressed());
        let text = client.transcribe_audio(&wav).await?;
        println!("{}", text);
    }
}

// Record audio data until stop returns true. returns the audio data as wav.
fn record(stop: impl Fn() -> bool) -> Vec<u8> {
    let host = cpal::default_host();
    let input_device = host.default_input_device().unwrap();

    let config = input_device.default_input_config().unwrap();
    let config: cpal::StreamConfig = config.into();

    let recording_state = Arc::new(Mutex::new(Vec::new()));
    let recording_state_clone = recording_state.clone();

    let input_stream = input_device
        .build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let mut recording_state = recording_state_clone.lock().unwrap();
                recording_state.extend_from_slice(data);
            },
            move |err| {
                eprintln!("An error occurred on the input audio stream: {}", err);
            },
            None,
        )
        .unwrap();

    input_stream.play().unwrap();

    while !stop() {
        std::thread::sleep(Duration::from_millis(16));
    }

    let audio_data: Vec<f32> = recording_state.lock().unwrap().clone();
    to_wav(audio_data, &config)
}
