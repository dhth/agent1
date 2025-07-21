use std::fmt::Display;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize)]
pub(super) struct GenerateContentBody {
    pub(super) contents: Vec<Content>,
    pub(super) system_instruction: SystemInstruction,
    pub(super) tools: Vec<Tool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct Content {
    pub(super) parts: Vec<Part>,
    pub(super) role: Option<Role>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct SystemInstruction {
    pub(super) parts: Vec<Part>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) enum Part {
    Text(String),
    FunctionCall(FunctionCall),
    FunctionResponse(FunctionResponse),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum Role {
    User,
    Model,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct GenerateContentResponse {
    pub(super) candidates: Vec<Candidate>,
    pub(super) usage_metadata: UsageMetadata,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(unused)]
pub(super) struct Candidate {
    pub(super) content: Content,
    pub(super) finish_reason: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(unused)]
pub(super) struct UsageMetadata {
    pub(super) prompt_token_count: usize,
    pub(super) candidates_token_count: usize,
    pub(super) total_token_count: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) enum Tool {
    FunctionDeclarations(Vec<FunctionDeclaration>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum FunctionDeclarationName {
    ReadFile,
    ListFiles,
    EditFile,
}

impl Display for FunctionDeclarationName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let content = match self {
            FunctionDeclarationName::ReadFile => "read_file",
            FunctionDeclarationName::ListFiles => "list_files",
            FunctionDeclarationName::EditFile => "edit_file",
        };

        write!(f, "{content}")
    }
}

#[derive(Debug, Serialize)]
pub(super) struct FunctionDeclaration {
    pub(super) name: FunctionDeclarationName,
    pub(super) description: String,
    pub(super) parameters: Value,
}

#[derive(Debug, JsonSchema, Deserialize)]
pub(super) struct ReadFileToolArgs {
    #[schemars(description = "The relative path of a file in the working directory")]
    pub(super) path: String,
}

#[derive(Debug, JsonSchema, Deserialize)]
pub(super) struct ListFileToolArgs {
    #[schemars(
        description = "Optional relative path to list files from. Defaults to current directory if not provided"
    )]
    pub(super) path: String,
}

#[derive(Debug, JsonSchema, Deserialize)]
pub(super) struct EditFileToolArgs {
    #[schemars(description = "The relative path of the file to edit")]
    pub(super) path: String,
    #[schemars(
        description = "Optional string to replace (if empty, replace the entire file contents)"
    )]
    pub(super) old_str: String,
    #[schemars(description = "The string to replace the old string with")]
    pub(super) new_str: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct FunctionCall {
    pub(super) id: Option<String>,
    pub(super) name: FunctionDeclarationName,
    pub(super) args: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct FunctionResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) id: Option<String>,
    pub(super) name: FunctionDeclarationName,
    pub(super) response: FunctionCallResponse,
    pub(super) will_continue: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum FunctionCallResponse {
    Output(String),
    Error(String),
}
