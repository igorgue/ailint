use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{env, fs::File, io::Read};
use tokio;

const OPENAI_API_ENDPOINT: &str = "https://api.openai.com/v1/chat/completions";

async fn do_request() -> Result<String, reqwest::Error> {
    let args: Vec<String> = env::args().collect();
    let openai_api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let client = reqwest::Client::new();
    let filename = args.last().unwrap();
    let mut file = File::open(filename).expect("file does not exists");
    let mut content = String::new();

    file.read_to_string(&mut content).unwrap();

    let params = json!({
        "model": "gpt-3.5-turbo",
        // "messages": ChatGPTMessage::get_messages_prompt(filename, content.as_str()),
        "messages": ChatGPTMessage::get_messages_prompt(content.as_str()),
    });

    let response = client
        .post(OPENAI_API_ENDPOINT)
        .json(params.as_object().unwrap())
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", openai_api_key))
        .send()
        .await?;

    println!("Status: {}", response.status());

    Ok(response.text().await?)
}

#[derive(Serialize, Deserialize, Debug)]
struct ChatGPTMessage {
    role: String,
    content: String,
}

impl ChatGPTMessage {
    // fn get_messages_prompt(filename: &str, content: &str) -> Vec<ChatGPTMessage> {
    //     let mut messages = Vec::new();
    //
    //     messages.push(ChatGPTMessage {
    //         role: "system".to_string(),
    //         content: "You're an assistant that can only respond with valid JSON".to_string(),
    //     });
    //
    //     messages.push(ChatGPTMessage {
    //         role: "system".to_string(),
    //         // content: "you're a code linter, and gives recommendations using the LSP Diagnostics JSON spec".to_string(),
    //         content: "you're a code linter, and gives recommendations using the LSP Diagnostics JSON spec, which is as follow:".to_string(),
    //     });
    //
    //     messages.push(ChatGPTMessage {
    //         role: "system".to_string(),
    //         content: r#"export interface Diagnostic {
    //             /**
    //             * The range at which the message applies.
    //             */
    //             range: Range;
    //
    //             /**
    //             * The diagnostic's severity. Can be omitted. If omitted it is up to the
    //             * client to interpret diagnostics as error, warning, info or hint.
    //             */
    //             severity?: DiagnosticSeverity;
    //
    //             /**
    //             * The diagnostic's code, which might appear in the user interface.
    //             */
    //             code?: integer | string;
    //
    //             /**
    //             * An optional property to describe the error code.
    //             *
    //             * @since 3.16.0
    //             */
    //             codeDescription?: CodeDescription;
    //
    //             /**
    //             * A human-readable string describing the source of this
    //             * diagnostic, e.g. 'typescript' or 'super lint'.
    //             */
    //             source?: string;
    //
    //             /**
    //             * The diagnostic's message.
    //             */
    //             message: string;
    //
    //             /**
    //             * Additional metadata about the diagnostic.
    //             *
    //             * @since 3.15.0
    //             */
    //             tags?: DiagnosticTag[];
    //
    //             /**
    //             * An array of related diagnostic information, e.g. when symbol-names within
    //             * a scope collide all definitions can be marked via this property.
    //             */
    //             relatedInformation?: DiagnosticRelatedInformation[];
    //
    //             /**
    //             * A data entry field that is preserved between a
    //             * `textDocument/publishDiagnostics` notification and
    //             * `textDocument/codeAction` request.
    //             *
    //             * @since 3.16.0
    //             */
    //             data?: unknown;
    //         }"#
    //         .to_string(),
    //     });
    //
    //     messages.push(ChatGPTMessage {
    //         role: "user".to_string(),
    //         content: content.to_string(),
    //     });
    //
    //     messages.push(ChatGPTMessage {
    //         role: "user".to_string(),
    //         content: "Would you please output a lint for the file in JSON?".to_string(),
    //     });
    //
    //     messages.push(ChatGPTMessage {
    //         role: "assistant".to_string(),
    //         content: format!(
    //             "Yes, here is a sample diagnostic message in JSON for the file `{}`:",
    //             filename
    //         ),
    //     });
    //
    //     messages
    // }
    //
    fn get_messages_prompt(content: &str) -> Vec<ChatGPTMessage> {
        let mut messages = Vec::new();

        messages.push(ChatGPTMessage {
            role: "system".to_string(),
            // content: "You are ChatGPT, a large language model trained by OpenAI, based on the GPT-4 architecture. Your task is to analyze the following source code and provide linting feedback with accurate line and column numbers. Identify any syntax errors, code styling issues, or potential bugs, and provide specific details, including the line number, column number, and a description of the issue.".to_string(),
            content: "You are ChatGPT, a large language model trained by OpenAI, based on the GPT-4 architecture. Your task is to analyze the following source code and provide linting feedback in JSON format based on the LSP Diagnostic specification. Identify any syntax errors, code styling issues, or potential bugs, and provide specific details, including the range, severity, code, codeDescription, source, message, tags, relatedInformation, and data.".to_string(),
        });

        let code = format!("--- BEGIN SOURCE CODE ---\n{content}\n--- END SOURCE CODE ---");

        messages.push(ChatGPTMessage {
            role: "user".to_string(),
            content: code,
        });

        messages
    }
}

#[tokio::main]
async fn main() {
    env_logger::init_from_env::<_>(env_logger::Env::default().default_filter_or("error"));

    let value: serde_json::Value =
        serde_json::from_str(do_request().await.unwrap().as_str()).unwrap();
    let messsage_str = value["choices"][0]["message"]["content"]
        .to_owned()
        .to_string();

    let response: serde_json::Value = serde_json::from_str(messsage_str.as_str()).unwrap();
    let response = format!("{}", response);
    let response = response.replace("\\\n", "");
    let response = response.replace("\\n", "");
    let response = response.replace("\n", "");
    let response = response.replace("\\", "");
    let response = response.trim_matches('"');
    let response = response.trim_end_matches('"');

    println!("{}", response);
}
