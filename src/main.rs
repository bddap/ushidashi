mod api;
mod audio;
mod button;
mod config;
mod consts;

use api::OpenAIApiClient;
use audio::record_wav;
use button::Button;
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
        println!("{}", text);
    }
}
