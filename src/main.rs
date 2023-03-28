mod api;
mod config;
mod prompt;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossterm::{
    event::{self, Event, KeyCode},
    terminal,
};
use hound::{WavSpec, WavWriter};
use std::io::{stdout, Write};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::api::OpenAIApiClient;

#[tokio::main]
async fn main() {
    match run().await {
        Ok(()) => (),
        Err(e) => eprintln!("Error: {:#?}", e),
    }
}

async fn run() -> anyhow::Result<()> {
    let secrets = config::Secrets::load()?;

    let client = OpenAIApiClient::new(&secrets.openai_api_key);

    let host = cpal::default_host();
    let input_device = host.default_input_device().unwrap();

    let config = input_device.default_input_config().unwrap();
    let config: cpal::StreamConfig = config.into();

    let recording_state = Arc::new(Mutex::new(Vec::new()));
    let is_recording = Arc::new(Mutex::new(false));

    let recording_state_clone = recording_state.clone();
    let is_recording_clone = is_recording.clone();

    let input_stream = input_device.build_input_stream(
        &config,
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            let mut recording_state = recording_state_clone.lock().unwrap();
            let is_recording = is_recording_clone.lock().unwrap();
            if *is_recording {
                recording_state.extend_from_slice(data);
            }
        },
        move |err| {
            eprintln!("An error occurred on the input audio stream: {}", err);
        },
        None,
    )?;

    input_stream.play()?;

    // Set up terminal
    let mut stdout = stdout();
    terminal::enable_raw_mode()?;
    stdout.write_all(b"Press and hold SPACE to start recording, release to transcribe")?;
    stdout.flush()?;

    loop {
        if event::poll(Duration::from_millis(10))? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Char(' ') => {
                        if key_event.modifiers.is_empty() {
                            *is_recording.lock().unwrap() = true;
                        }
                    }
                    KeyCode::Esc => {
                        break;
                    }
                    _ => {}
                }
            }
        } else {
            if *is_recording.lock().unwrap() {
                let audio_data: Vec<f32> = recording_state.lock().unwrap().clone();
                let wav = to_wav(audio_data, &config);
                let transcription = client.transcribe_audio(&wav).await?;
                println!("\nTranscription: {}", transcription);

                *is_recording.lock().unwrap() = false;
                *recording_state.lock().unwrap() = Vec::new();
            }
        }
    }

    // Clean up terminal
    stdout.write_all(b"\n")?;
    stdout.flush()?;
    terminal::disable_raw_mode()?;

    Ok(())
}

fn to_wav(audio_data_f32: Vec<f32>, config: &cpal::StreamConfig) -> Vec<u8> {
    // Convert f32 samples to i16
    let audio_data_i16: Vec<i16> = audio_data_f32
        .iter()
        .map(|sample| (sample * i16::MAX as f32) as i16)
        .collect();

    // Compress the audio data to WAV
    let spec = WavSpec {
        channels: config.channels,
        sample_rate: config.sample_rate.0,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut buffer = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buffer);
    {
        let mut writer = WavWriter::new(&mut cursor, spec).unwrap();
        for &sample in &audio_data_i16 {
            writer.write_sample(sample).unwrap();
        }
    }

    buffer
}
