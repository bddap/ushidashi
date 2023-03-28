use hound::{WavSpec, WavWriter};

pub fn to_wav(audio_data_f32: Vec<f32>, config: &cpal::StreamConfig) -> Vec<u8> {
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
