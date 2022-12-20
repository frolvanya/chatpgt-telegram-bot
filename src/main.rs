use std::env;

use anyhow::{anyhow, Context, Result};

use futures::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use reqwest::{blocking::Client, header};
use telegram_bot::*;

use log::{error, info};

#[derive(Debug, Serialize, Deserialize)]
struct CompletionsApiJson {
    model: String,
    prompt: String,
    max_tokens: u16,
    temperature: f32,
}

fn chatgpt(api: String, prompt: &str) -> Result<String> {
    let url = "https://api.openai.com/v1/completions";

    let data = CompletionsApiJson {
        model: "text-davinci-003".to_string(),
        prompt: prompt.into(),
        max_tokens: 4000,
        temperature: 1.0,
    };

    let client = Client::new();
    let response = match client
        .post(url)
        .header(header::AUTHORIZATION, format!("Bearer {api}"))
        .json(&data)
        .send()
    {
        Ok(res) => res,
        Err(e) => {
            error!("Failed to send POST request to `{url}`");
            return Err(anyhow!(e));
        }
    };

    let response_text = match response.text() {
        Ok(res) => res,
        Err(e) => {
            error!("Failed to get response text");
            return Err(anyhow!(e));
        }
    };

    let response_json: Value = match serde_json::from_str(&response_text) {
        Ok(res) => res,
        Err(e) => {
            error!("Failed to parse response text as JSON, due to {e}");
            return Err(anyhow!(e));
        }
    };
    if response_json.get("error") != None {
        return Ok(response_json["error"]["message"].to_string());
    }

    Ok(response_json["choices"][0]["text"].to_string())
}

#[tokio::main]
async fn main() -> Result<()> {
    log4rs::init_file("log_config.yaml", Default::default())
        .context("Failed to load config for logging file")?;

    let chatgpt_api = env::var("CHATGPT_API").context("Set up `CHATGPT_API` first")?;
    let telegram_api = env::var("TELEGRAM_BOT_API").context("Set up `TELEGRAM_BOT_API` first")?;

    let api = Api::new(telegram_api);

    let mut stream = api.stream();
    while let Some(update) = stream.next().await {
        let update = update?;

        if let UpdateKind::Message(message) = update.kind {
            if let MessageKind::Text { ref data, .. } = message.kind {
                info!(
                    "<{}>: \"{}\"",
                    &message.from.first_name,
                    message.text().unwrap()
                );

                let reply_text = match chatgpt(chatgpt_api.clone(), data) {
                    Ok(text) => String::from(
                        &text
                            .trim_start_matches(&[
                                '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '{', '}', '[',
                                ']', '|', '\\', '\'', '"', ':', ';', ',', '<', '.', '>', '/', '?',
                                'n',
                            ])
                            .trim_end_matches('"')
                            .replace("\\n", "\n"),
                    ),
                    Err(_) => {
                        String::from("ChatGPT API currently is not working. Try again later...")
                    }
                };

                match api.send(message.text_reply(reply_text.clone())).await {
                    Ok(_) => info!("<ChatPGT>: \"{reply_text}\""),
                    Err(e) => error!("Failed to send telegram message, due to {e}"),
                };
            }
        }
    }
    Ok(())
}
