use super::domain::*;
use crate::tools::{edit_file, list_files, read_file, run_cmd};
use anyhow::Context;
use schemars::schema_for;
use serde_json::Value;

pub(super) fn get_tools() -> Vec<Tool> {
    let read_file_tool_schema = schema_for!(ReadFileToolArgs);
    let mut read_file_tool_schema_value: Value = read_file_tool_schema.to_value();

    if let Value::Object(ref mut obj) = read_file_tool_schema_value {
        obj.remove("$schema");
        obj.remove("title");
    }

    let read_file_tool = Tool::FunctionDeclarations (
        vec![FunctionDeclaration {
            name: FunctionDeclarationName::ReadFile,
            description: "Read the contents of a given relative file path. Use this when you want to see what's inside a file. Do not use this with directory names.".to_string(),
            parameters: read_file_tool_schema_value,
        }],
    );

    let list_files_tool_schema = schema_for!(ListFileToolArgs);
    let mut list_files_tool_schema_value: Value = list_files_tool_schema.to_value();

    if let Value::Object(ref mut obj) = list_files_tool_schema_value {
        obj.remove("$schema");
        obj.remove("title");
    }

    let list_files_tool = Tool::FunctionDeclarations (
        vec![FunctionDeclaration {
            name: FunctionDeclarationName::ListFiles,
            description: "List files and directories at a given path. If no path is provided, lists files in the current directory.".to_string(),
            parameters: list_files_tool_schema_value,
        }],
    );

    let edit_file_tool_schema = schema_for!(EditFileToolArgs);
    let mut edit_file_tool_schema_value: Value = edit_file_tool_schema.to_value();

    if let Value::Object(ref mut obj) = edit_file_tool_schema_value {
        obj.remove("$schema");
        obj.remove("title");
    }

    let edit_file_tool = Tool::FunctionDeclarations (
        vec![FunctionDeclaration {
            name: FunctionDeclarationName::EditFile,
            description: r#"Make edits to a text file.

Replaces 'old_str' with 'new_str' in the given file. 'old_str' and 'new_str' MUST be different from each other.

If 'old_str' is empty, the entire file will be replaced with 'new_str' (use this for editing an entire file).
If the file specified with path doesn't exist, it will be created."#.to_string(),
            parameters: edit_file_tool_schema_value,
        }],
    );

    let run_command_tool_schema = schema_for!(RunCmdArgs);
    let mut run_command_tool_schema_value: Value = run_command_tool_schema.to_value();

    if let Value::Object(ref mut obj) = run_command_tool_schema_value {
        obj.remove("$schema");
        obj.remove("title");
    }

    let run_cmd_tool = Tool::FunctionDeclarations(vec![FunctionDeclaration {
        name: FunctionDeclarationName::RunCmd,
        description: "Run a shell command via bash. Will return the combined stdout and stderr of the command.".to_string(),
        parameters: run_command_tool_schema_value,
    }]);

    vec![
        read_file_tool,
        list_files_tool,
        edit_file_tool,
        run_cmd_tool,
    ]
}

pub(super) fn execute_function_call(
    function_call: &FunctionCall,
) -> anyhow::Result<FunctionResponse> {
    let response = match &function_call.name {
        FunctionDeclarationName::ReadFile => {
            let args = match &function_call.args {
                Some(a) => a,
                None => {
                    return Err(anyhow::anyhow!("empty args provided"));
                }
            };

            let args: ReadFileToolArgs =
                serde_json::from_value(args.clone()).context("invalid arguments provided")?;
            match read_file(&args.path) {
                Ok(r) => FunctionCallResponse::Output(r),
                Err(e) => FunctionCallResponse::Error(e.to_string()),
            }
        }
        FunctionDeclarationName::ListFiles => {
            let path = match &function_call.args {
                Some(a) => {
                    let args: ListFileToolArgs =
                        serde_json::from_value(a.clone()).context("invalid arguments provided")?;
                    if args.path.is_empty() {
                        None
                    } else {
                        Some(args.path)
                    }
                }
                None => None,
            };

            match list_files(path.as_deref().unwrap_or(".")) {
                Ok(r) => FunctionCallResponse::Output(r.join(", ")),
                Err(e) => FunctionCallResponse::Error(e.to_string()),
            }
        }
        FunctionDeclarationName::EditFile => {
            let args = match &function_call.args {
                Some(a) => a,
                None => {
                    return Err(anyhow::anyhow!("empty args provided"));
                }
            };

            let args: EditFileToolArgs =
                serde_json::from_value(args.clone()).context("invalid arguments provided")?;
            match edit_file(&args.path, &args.old_str, &args.new_str) {
                Ok(r) => FunctionCallResponse::Output(r),
                Err(e) => FunctionCallResponse::Error(e.to_string()),
            }
        }
        FunctionDeclarationName::RunCmd => {
            let args = match &function_call.args {
                Some(a) => a,
                None => {
                    return Err(anyhow::anyhow!("empty args provided"));
                }
            };

            let args: RunCmdArgs =
                serde_json::from_value(args.clone()).context("invalid arguments provided")?;
            match run_cmd(&args.cmd) {
                Ok(r) => FunctionCallResponse::Output(r),
                Err(e) => FunctionCallResponse::Error(e.to_string()),
            }
        }
    };

    Ok(FunctionResponse {
        id: function_call.id.clone(),
        name: function_call.name.clone(),
        response,
        will_continue: Some(false),
    })
}
