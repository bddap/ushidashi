mod api;
mod audio;
mod button;
mod chatlog;
mod config;
mod consts;

use api::{ChatCompletionRequest, Message, OpenAIApiClient};
use audio::record_wav;
use button::Button;
use chatlog::{Author, LogMessage};
use consts::{POLL_INTERVAL, SYSTEM_PROMPT};

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
        let wav = record_wav(&button);
        let text = client.transcribe_audio(&wav).await?;
        let _next_mesage = get_response(&client, &text).await?;
    }
}

async fn get_response(client: &OpenAIApiClient, prompt: &str) -> anyhow::Result<String> {
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
    let mut response = client.get_completion(request).await?;

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
