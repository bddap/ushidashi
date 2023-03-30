use reqwest::{multipart, Client};
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
struct TranscriptionResponse {
    text: String,
}

pub struct OpenAIApiClient {
    client: Client,
    api_key: String,
}

impl OpenAIApiClient {
    pub fn new(api_key: &str) -> Self {
        OpenAIApiClient {
            client: Client::new(),
            api_key: api_key.to_string(),
        }
    }

    pub async fn transcribe_audio(&self, audio_data: &[u8]) -> anyhow::Result<String> {
        let model = "whisper-1";
        let language = "en";
        let url = "https://api.openai.com/v1/audio/transcriptions";
        let part = multipart::Part::bytes(audio_data.to_vec()).file_name("audio.wav");
        let form = multipart::Form::new()
            .part("file", part)
            .text("model", model.to_string())
            .text("language", language);

        let res = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .multipart(form)
            .send()
            .await?;

        let status = res.status();
        let body: Value = res.json().await?;
        let pretty_body = serde_json::to_string_pretty(&body)?;

        anyhow::ensure!(
            status.is_success(),
            "OpenAI API non success. status: {status}\nbody: {pretty_body}",
        );

        let transcription_response: TranscriptionResponse = serde_json::from_value(body)
            .map_err(|e| anyhow::anyhow!("Error: {:?}\nBody {:?}", e, pretty_body))?;

        Ok(transcription_response.text)
    }
}
