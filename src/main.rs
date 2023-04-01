mod audio;
mod button;
mod chatlog;
mod config;
mod consts;
mod google_tts;
mod openai;

use audio::{play_wav, record_wav};
use button::Button;
use chatlog::{Author, LogMessage};
use consts::{POLL_INTERVAL, SYSTEM_PROMPT};
use google_tts::{Input::Ssml, TtsClient};
use openai::{ChatCompletionRequest, Message, OpenAIApiClient};

#[tokio::main]
async fn main() {
    match run().await {
        Ok(()) => (),
        Err(e) => {
            eprintln!("Error: {:#?}", e);
            eprintln!("Error: {}", e);
        }
    }
}

async fn run() -> anyhow::Result<()> {
    eprintln!("chatlog location: {:?}", chatlog::logfile()?);

    let secrets = config::Secrets::load()?;

    let openai = OpenAIApiClient::new(&secrets.openai_api_key);
    let tts = TtsClient::new(&secrets.google_tts_api_key);

    let button = Button::create();

    loop {
        while !button.pressed().ok_or(anyhow::anyhow!("button closed"))? {
            std::thread::sleep(POLL_INTERVAL);
        }
        let wav = record_wav(&button);
        let text = openai.transcribe_audio(&wav).await?;
        let next_message = get_response(&openai, &text).await?;
        let wav = tts.synthesize(Ssml(next_message)).await?;
        play_wav(&wav)?;
    }
}

async fn get_response(openai: &OpenAIApiClient, prompt: &str) -> anyhow::Result<String> {
    // prefix the prompt with a timestamp
    let prompt = format!("{}\n{}", chrono::Local::now(), prompt);

    let mut messages = get_history()?;

    chatlog::store_message(LogMessage::user(prompt.clone()))?;

    messages.push(Message::user(prompt));

    let request = ChatCompletionRequest {
        model: "gpt-4".into(),
        messages,
        max_tokens: None,
        temperature: None,
        top_p: None,
        presence_penalty: None,
        frequency_penalty: None,
        stop: None,
        n: None,
        stream: None,
        logit_bias: None,
        user: None,
    };
    let mut response = openai.get_completion(request).await?;

    anyhow::ensure!(
        response.choices.len() == 1,
        "Expected exactly one choice, got {}",
        response.choices.len()
    );

    let ret = response.choices.remove(0).message.content;

    chatlog::store_message(LogMessage::bot(ret.clone()))?;

    Ok(ret)
}

/// load entire conversation history, including the system prompt
fn get_history() -> anyhow::Result<Vec<Message>> {
    let mut ret = [Message::system(SYSTEM_PROMPT)].to_vec();
    for log_message in chatlog::load_messages()? {
        let LogMessage { author, text } = log_message;
        let message = match author {
            Author::User => Message::user(text),
            Author::Bot => Message::system(text),
        };
        ret.push(message);
    }
    Ok(ret)
}
