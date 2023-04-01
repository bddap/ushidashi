use base64::prelude::{Engine, BASE64_STANDARD};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub struct TtsClient {
    api_key: String,
    client: Client,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct SynthesizeRequest {
    input: Input,
    voice: Voice,
    audio_config: AudioConfig,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Input {
    text: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Voice {
    language_code: String,
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct AudioConfig {
    audio_encoding: String,
    speaking_rate: f64,
    pitch: i32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct SynthesizeResponse {
    audio_content: String,
}

impl TtsClient {
    pub fn new(api_key: &str) -> TtsClient {
        TtsClient {
            api_key: api_key.to_string(),
            client: Client::new(),
        }
    }

    pub async fn synthesize(&self, input_text: &str) -> anyhow::Result<Vec<u8>> {
        let url = format!(
            "https://texttospeech.googleapis.com/v1/text:synthesize?key={}",
            self.api_key
        );

        let request_body = SynthesizeRequest {
            input: Input {
                text: input_text.to_string(),
            },
            voice: Voice {
                language_code: "en-US".to_string(),
                name: "en-US-Wavenet-A".to_string(),
            },
            audio_config: AudioConfig {
                audio_encoding: "LINEAR16".to_string(),
                speaking_rate: 1.0,
                pitch: 0,
            },
        };

        let res = self.client.post(&url).json(&request_body).send().await?;

        let status = res.status();
        let body: Value = res.json().await?;
        let pretty_body = serde_json::to_string_pretty(&body)?;

        anyhow::ensure!(
            status.is_success(),
            "Tts API non success. status: {status}\nbody: {pretty_body}",
        );

        let response: SynthesizeResponse = serde_json::from_value(body).map_err(|e| {
            anyhow::anyhow!("Failed to parse response. body: {pretty_body}\nerror: {e}")
        })?;

        // Decode the Base64 string to get a Vec<u8> with the WAV data
        let audio = BASE64_STANDARD.decode(response.audio_content.as_bytes())?;

        Ok(audio)
    }
}
