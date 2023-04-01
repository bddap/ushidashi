use std::{
    collections::VecDeque,
    sync::{mpsc, Arc, Mutex},
};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound::{WavSpec, WavWriter};

use crate::{button::Button, consts::POLL_INTERVAL};

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

pub fn record_wav(button: &Button) -> Vec<u8> {
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

pub fn play_wav(wav: &[u8]) -> anyhow::Result<()> {
    let host = cpal::default_host();
    let output_device = host
        .default_output_device()
        .ok_or(anyhow::anyhow!("No default output device found"))?;

    let mut reader = hound::WavReader::new(std::io::Cursor::new(wav))?;
    let spec = reader.spec();

    let config = cpal::StreamConfig {
        channels: spec.channels,
        sample_rate: cpal::SampleRate(spec.sample_rate),
        buffer_size: cpal::BufferSize::Default,
    };

    let mut audio_data = VecDeque::new();
    for sample in reader.samples::<i16>() {
        audio_data.push_back(sample? as f32 / i16::MAX as f32);
    }

    let (tx, rx) = mpsc::channel::<anyhow::Result<()>>();
    let tx_clone = tx.clone();

    let output_stream = output_device
        .build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                for sample in data.iter_mut() {
                    *sample = audio_data.pop_front().unwrap_or(0.0);
                }
                if audio_data.is_empty() {
                    let _ = tx.send(Ok(()));
                }
            },
            move |err| {
                let _ = tx_clone.send(Err(anyhow::anyhow!(err)));
            },
            None,
        )
        .map_err(|e| anyhow::anyhow!("Failed to build audio output stream: {}", e))?;

    output_stream
        .play()
        .map_err(|e| anyhow::anyhow!("Failed to play audio output stream: {}", e))?;

    rx.recv()?
}
