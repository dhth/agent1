use super::domain::*;
use super::tools::{execute_function_call, get_tools};
use anyhow::Context;
use colored::Colorize;
use reqwest::blocking::Client;
use std::env::VarError;
use std::io::Write;
use tracing::debug;

const SYSTEM_PROMPT: &str = include_str!("assets/system-prompt.txt");

pub fn run(client: Client) -> anyhow::Result<()> {
    let api_key = match std::env::var("GEMINI_API_KEY") {
        Ok(k) => k,
        Err(e) => match e {
            VarError::NotPresent => {
                return Err(anyhow::anyhow!(
                    "environment variable GEMINI_API_KEY not set"
                ));
            }
            VarError::NotUnicode(_) => {
                return Err(anyhow::anyhow!(
                    "environment variable GEMINI_API_KEY is not valid unicode"
                ));
            }
        },
    };

    println!(
        "{}",
        r#"
                 _   ___
 ___ ___ ___ ___| |_|_  |
| .'| . | -_|   |  _|_| |_
|__,|_  |___|_|_|_| |_____|
    |___|
"#
        .blue()
    );
    println!("{}", "Commands:".blue());
    println!("{}", "/quit or /exit or /bye to quit".yellow());
    println!("{}", "/new to start a new session".yellow());
    println!("{}", "/clear to clear the screen".yellow());
    println!();

    let mut read_user_input = true;
    let tools = get_tools();

    let mut body = GenerateContentBody {
        contents: vec![],
        tools,
        system_instruction: SystemInstruction {
            parts: vec![Part::Text(SYSTEM_PROMPT.to_string())],
        },
    };

    let mut total_token_count = 0;

    loop {
        if read_user_input {
            print!("{}", "You: ".green());
            let mut user_input = String::new();
            std::io::stdout().flush().unwrap();
            std::io::stdin()
                .read_line(&mut user_input)
                .context("couldn't read user input")?;

            let user_input = user_input.trim().to_string();

            match user_input.as_str() {
                "/bye" | "/quit" | "/exit" => break,
                "/new" => {
                    body.contents = vec![];
                    total_token_count = 0;
                    // TODO: make this cross platform
                    print!("\x1B[2J\x1B[1;1H");
                    std::io::stdout().flush().context("couldn't clear screen")?;
                    println!("{}", "context cleared".dimmed());
                    continue;
                }
                "/clear" => {
                    // TODO: make this cross platform
                    print!("\x1B[2J\x1B[1;1H");
                    std::io::stdout().flush().context("couldn't clear screen")?;
                    continue;
                }
                _ => {}
            }

            body.contents.push(Content {
                parts: vec![Part::Text(user_input)],
                role: Some(Role::User),
            });
        }

        if let Ok(r) = serde_json::to_string_pretty(&body) {
            debug!("request: {}", &r);
        } else {
            debug!("request: {:?}", &body);
        }

        let resp = client.post("https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent")
        .json(&body)
        .header("content-type", "application/json")
        .header("x-goog-api-key", &api_key)
        .send();

        let resp = match resp {
            Ok(r) => r,
            Err(e) => {
                println!(
                    "{}",
                    format!("couldn't send a request to Gemini API: {e}").red()
                );
                continue;
            }
        };

        if !resp.status().is_success() {
            let code = resp.status().as_u16();
            match resp.text() {
                Ok(t) => debug!("non success response (code: {code}): {}", &t),
                Err(e) => debug!("non success response (code: {code}), couldn't get text: {e}",),
            }
            anyhow::bail!("gemini API returned a non success code: {code}");
        }

        let resp_body = resp
            .text()
            .inspect_err(|e| debug!("couldn't get response text: {e}"))
            .context("couldn't get response text")?;
        debug!("response: {}", &resp_body);

        let resp = match serde_json::from_str::<GenerateContentResponse>(&resp_body) {
            Ok(r) => r,
            Err(e) => {
                println!(
                    "{}",
                    format!("couldn't parse response from Gemini: {e}").red()
                );
                continue;
            }
        };

        total_token_count += resp.usage_metadata.total_token_count;

        let content = match resp.candidates.into_iter().next() {
            Some(c) => c.content,
            None => {
                println!("{}", "gemini returned an empty response".dimmed());
                continue;
            }
        };

        let mut function_results: Vec<Part> = vec![];
        for part in &content.parts {
            match part {
                Part::Text(t) => {
                    println!("{}: {}", "Gemini".blue(), t.trim());
                }
                Part::FunctionCall(call) => {
                    println!("{}: wants to call function {}", "Gemini".blue(), call.name);
                    let function_resp =
                        execute_function_call(call).unwrap_or_else(|e| FunctionResponse {
                            id: call.id.clone(),
                            name: call.name.clone(),
                            response: FunctionCallResponse::Error(e.to_string()),
                            will_continue: None,
                        });

                    function_results.push(Part::FunctionResponse(function_resp));
                }
                _ => {}
            }
        }
        body.contents.push(content);

        if total_token_count > 0 {
            println!(
                "{}",
                format!("total tokens used: {total_token_count}").dimmed()
            );
        }

        if function_results.is_empty() {
            read_user_input = true;
            println!("{}", "---".dimmed());
            continue;
        }

        body.contents.push(Content {
            parts: function_results,
            role: Some(Role::User),
        });

        read_user_input = false;
        println!("{}", "---".dimmed());
    }

    Ok(())
}
