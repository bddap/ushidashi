mod api;
mod audio;
mod button;
mod config;
mod system_prompt;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::api::OpenAIApiClient;
use crate::audio::to_wav;
use crate::button::Button;

const POLL_INTERVAL: Duration = Duration::from_millis(16);

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

    let button = Button::create();

    loop {
        while !button.pressed().ok_or(anyhow::anyhow!("button closed"))? {
            std::thread::sleep(POLL_INTERVAL);
        }
        let wav = record(&button);
        let text = client.transcribe_audio(&wav).await?;
        println!("{}", text);
    }
}

fn record(button: &Button) -> Vec<u8> {
    let host = cpal::default_host();
    let input_device = host.default_input_device().unwrap();

    let config = input_device.default_input_config().unwrap();
    let config: cpal::StreamConfig = config.into();

    let recording_state = Arc::new(Mutex::new(Some(Vec::new())));
    let recording_state_clone = recording_state.clone();

    let input_stream = input_device
        .build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let mut recording_state = recording_state_clone.lock().unwrap();
                if let Some(ref mut recording_state) = *recording_state {
                    recording_state.extend_from_slice(data);
                }
            },
            move |err| {
                eprintln!("An error occurred on the input audio stream: {}", err);
            },
            None,
        )
        .unwrap();

    input_stream.play().unwrap();

    while button.pressed().unwrap_or(false) {
        std::thread::sleep(POLL_INTERVAL);
    }

    let audio_data: Vec<f32> = recording_state.lock().unwrap().take().unwrap();
    to_wav(audio_data, &config)
}
