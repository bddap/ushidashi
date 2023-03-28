use reqwest::{multipart, Client, Error};
use serde::Deserialize;

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

    pub async fn transcribe_audio(&self, audio_data: &[u8]) -> Result<String, Error> {
        let model = "whisper-1";
        let language = "en-US";
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
            .await?
            .error_for_status()?;

        let transcription_response: TranscriptionResponse = res.json().await?;
        Ok(transcription_response.text)
    }
}
