use super::core::{
    AvailableTool, HttpMethod, ModelParameter, TextOperation, ExportFormat,
    CargoOperation, NpmOperation, PipOperation, DockerResourceType,
};
use colored::Colorize;
use serde::{Deserialize, Serialize};
// use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
struct ToolAnalysis {
    reasoning: String,
    tools: Vec<ToolRequest>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ToolRequest {
    tool_type: String,
    parameters: serde_json::Value,
    reasoning: String,
}

pub struct NaturalLanguageParser {
    // Enhanced with model awareness
}

impl NaturalLanguageParser {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn parse_request_with_llm(
        &self,
        input: &str,
        llm_client: &crate::client::SelectedModel,
    ) -> Vec<AvailableTool> {
        // First check for immediate commands that don't need LLM parsing
        if let Some(tool) = self.parse_immediate_commands(input) {
            return vec![tool];
        }

        let analysis_prompt = self.build_enhanced_analysis_prompt(input);

        // Get LLM response
        match crate::client::stream_response(llm_client, &analysis_prompt).await {
            Ok(response) => {
                if let Ok(analysis) = self.parse_llm_response(&response) {
                    return self.convert_to_tools(analysis);
                }
            }
            Err(e) => {
                println!("{} Error getting LLM analysis: {}", "⚠".yellow(), e);
            }
        }

        // Enhanced fallback with more sophisticated parsing
        self.enhanced_fallback_parse(input)
    }

    fn parse_immediate_commands(&self, _input: &str) -> Option<AvailableTool> {
        // Remove all hardcoded patterns - let LLM handle everything
        None
    }

    fn build_enhanced_analysis_prompt(&self, user_input: &str) -> String {
        format!(
            r#"You are an intelligent command parser that analyzes user requests and maps them to the appropriate tools.

USER REQUEST: "{}"

AVAILABLE TOOLS AND THEIR USAGE PATTERNS:

## File Operations
- FileRead: Read file content
  Examples: "read Cargo.toml", "show main.rs", "what's in the config file", "display package.json"
  Parameters: path (string, exact filename)

- FileWrite: Write content to file
  Examples: "write hello to test.txt", "create readme with content", "save data to file.json"
  Parameters: path (string), content (string)

- FileEdit: Edit existing file
  Examples: "edit main.rs", "modify config", "update the dockerfile"
  Parameters: path (string), operation (object)

- FileSearch: Find files by pattern
  Examples: "find *.rs files", "search for config files", "locate all json files"
  Parameters: pattern (string), directory (optional string)

- ContentSearch: Search text within files
  Examples: "find TODO in code", "search for main function", "look for error messages"
  Parameters: pattern (string), directory (optional string)

- ListDirectory: List directory contents
  Examples: "list files", "show directory", "what's in src/", "ls"
  Parameters: path (string)

- FileWatch: Monitor file changes
  Examples: "watch config.json", "monitor Cargo.toml for 60 seconds", "watch the main.rs file for changes", "observe package.json for 2 minutes"
  Parameters: path (string, exact filename), duration_seconds (optional number, convert minutes to seconds)
  IMPORTANT: For "watch the X" format, extract X as the path, not "the"

## Git Operations
- GitStatus: Check repository status
  Examples: "git status", "check git", "repo status", "show changes"
  Parameters: repository_path (optional string - OMIT unless user specifies a specific directory)

- GitAdd: Stage files
  Examples: "git add main.rs", "stage changes", "add all files", "stage everything"
  Parameters: files (array of strings), repository_path (optional string - OMIT unless user specifies a specific directory)

- GitCommit: Create commit
  Examples: "commit changes", "git commit with message fix bug", "commit 'added feature'"
  Parameters: message (string), repository_path (optional string - OMIT unless user specifies a specific directory)

- GitPush: Push to remote
  Examples: "push changes", "git push", "push to origin", "push to main branch"
  Parameters: remote (optional string), branch (optional string), repository_path (optional string - OMIT unless user specifies a specific directory)

- GitPull: Pull from remote
  Examples: "pull changes", "git pull", "pull from origin", "update from remote"
  Parameters: remote (optional string), branch (optional string), repository_path (optional string - OMIT unless user specifies a specific directory)

- GitLog: Show commit history
  Examples: "git log", "show commits", "last 5 commits", "commit history"
  Parameters: count (optional number), oneline (boolean), repository_path (optional string - OMIT unless user specifies a specific directory)

## System Operations
- SystemInfo: Get system information
  Examples: "system info", "system details", "show system", "hardware info"
  Parameters: none

- MemoryUsage: Check memory usage
  Examples: "memory usage", "check memory", "show ram", "memory info"
  Parameters: none

- DiskUsage: Check disk space
  Examples: "disk usage", "check disk space", "storage info", "disk space in /home"
  Parameters: path (optional string)

- ProcessList: List running processes
  Examples: "list processes", "show processes", "running apps", "ps aux"
  Parameters: filter (optional string)

- ExecuteCommand: Run system commands
  Examples: "run ls -la", "execute python script.py", "command mkdir test"
  Parameters: command (string)

## Model Configuration
- SetModelParameter: Change model settings
  Examples: "set temperature to 0.8", "change max tokens to 2048", "set top-p to 0.9"
  Parameters: parameter (enum), value (varies by parameter)

- GetModelParameter: View model settings
  Examples: "show model config", "get temperature", "display settings", "model parameters"
  Parameters: parameter (optional enum)

- SwitchModel: Change active model
  Examples: "switch to llama2", "use codellama", "change model to gemma", "switch model"
  Parameters: model_name (string)

## Package Management
- CargoOperation: Rust operations
  Examples: "cargo build", "cargo test", "add serde", "build project"
  Parameters: operation (enum), package (optional string), features (optional array)

- NpmOperation: Node.js operations
  Examples: "npm install", "npm run dev", "install express", "run tests"
  Parameters: operation (enum), package (optional string), dev (boolean)

## Web & API
- WebSearch: Search internet
  Examples: "search rust tutorials", "google python guides", "find documentation"
  Parameters: query (string)

- WebScrape: Extract web content
  Examples: "scrape https://example.com", "get content from url", "extract webpage"
  Parameters: url (string)

- HttpRequest: Make HTTP requests
  Examples: "GET api.example.com", "POST to webhook", "HTTP request to server"
  Parameters: method (enum), url (string), headers (optional object), body (optional string)

## Text Processing
- JsonFormat: Format JSON
  Examples: "format json", "pretty print json", "beautify json data"
  Parameters: input (string)

- RegexMatch: Pattern matching
  Examples: "find emails in text", "match phone numbers", "extract urls"
  Parameters: pattern (string), text (string), flags (optional string)

## Session Management
- ClearHistory: Clear conversation
  Examples: "clear history", "clear conversation", "reset chat", "new session"
  Parameters: none

PARSING RULES:
1. Understand user intent, not just keywords
2. Handle natural language variations and synonyms
3. Extract parameters intelligently from context
4. Convert time units (1 minute = 60 seconds)
5. Preserve exact case for filenames (Cargo.toml, not cargo.toml)
6. For "watch the X" format, the path is X, not "the"
7. Use sensible defaults for optional parameters
8. Handle multiple tools if the request is complex

RESPONSE FORMAT (JSON only):
{{
  "reasoning": "What the user wants to accomplish",
  "tools": [
    {{
      "tool_type": "ToolName",
      "parameters": {{
        "param_name": "value"
      }},
      "reasoning": "Why this tool is needed"
    }}
  ]
}}

Analyze the request and respond with JSON only:"#,
            user_input
        )
    }

    fn parse_llm_response(&self, response: &str) -> Result<ToolAnalysis, serde_json::Error> {
        // Try multiple strategies to extract JSON from reasoning model responses
        
        // Strategy 1: Look for complete JSON objects with proper brace matching
        if let Some(json_str) = self.extract_json_with_brace_matching(response) {
            if let Ok(analysis) = serde_json::from_str::<ToolAnalysis>(&json_str) {
                return Ok(analysis);
            }
        }
        
        // Strategy 2: Look for JSON blocks marked with ```json
        if let Some(json_str) = self.extract_json_from_code_blocks(response) {
            if let Ok(analysis) = serde_json::from_str::<ToolAnalysis>(&json_str) {
                return Ok(analysis);
            }
        }
        
        // Strategy 3: Original simple extraction (fallback)
        if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                let json_str = &response[start..=end];
                if let Ok(analysis) = serde_json::from_str::<ToolAnalysis>(json_str) {
                    return Ok(analysis);
                }
            }
        }

        // Strategy 4: Try parsing the entire response as JSON
        serde_json::from_str(response)
    }
    
    fn extract_json_with_brace_matching(&self, response: &str) -> Option<String> {
        let mut brace_count = 0;
        let mut start_pos = None;
        let chars: Vec<char> = response.chars().collect();
        
        for (i, &ch) in chars.iter().enumerate() {
            match ch {
                '{' => {
                    if brace_count == 0 {
                        start_pos = Some(i);
                    }
                    brace_count += 1;
                }
                '}' => {
                    brace_count -= 1;
                    if brace_count == 0 {
                        if let Some(start) = start_pos {
                            let json_str = &response[start..=i];
                            // Quick validation that this looks like our expected structure
                            if json_str.contains("\"reasoning\"") && json_str.contains("\"tools\"") {
                                return Some(json_str.to_string());
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        
        None
    }
    
    fn extract_json_from_code_blocks(&self, response: &str) -> Option<String> {
        // Look for ```json ... ``` blocks
        if let Some(start) = response.find("```json") {
            let content_start = start + 7;
            if let Some(end) = response[content_start..].find("```") {
                let json_str = &response[content_start..content_start + end].trim();
                return Some(json_str.to_string());
            }
        }
        
        // Look for ``` ... ``` blocks that might contain JSON
        if let Some(start) = response.find("```") {
            let content_start = start + 3;
            if let Some(end) = response[content_start..].find("```") {
                let potential_json = &response[content_start..content_start + end].trim();
                if potential_json.starts_with('{') && potential_json.ends_with('}') {
                    return Some(potential_json.to_string());
                }
            }
        }
        
        None
    }

    fn convert_to_tools(&self, analysis: ToolAnalysis) -> Vec<AvailableTool> {
        let mut tools = Vec::new();

        println!(
            "{} LLM Analysis: {}",
            "🧠".cyan(),
            analysis.reasoning.blue()
        );

        for tool_req in analysis.tools {
            println!(
                "  {} {} - {}",
                "→".blue(),
                tool_req.tool_type.yellow(),
                tool_req.reasoning.dimmed()
            );

            match tool_req.tool_type.as_str() {
                // Existing tools
                "FileRead" => {
                    let path = tool_req.parameters.get("path")
                        .or_else(|| tool_req.parameters.get("filename"))
                        .or_else(|| tool_req.parameters.get("file"))
                        .or_else(|| tool_req.parameters.get("filepath"))
                        .and_then(|v| v.as_str());
                    
                    if let Some(path) = path {
                        tools.push(AvailableTool::FileRead {
                            path: path.to_string(),
                        });
                    }
                }
                "FileWrite" => {
                    let path = tool_req.parameters.get("path")
                        .or_else(|| tool_req.parameters.get("filename"))
                        .or_else(|| tool_req.parameters.get("file"))
                        .or_else(|| tool_req.parameters.get("filepath"))
                        .and_then(|v| v.as_str());
                    
                    if let Some(path) = path {
                        let content = tool_req
                            .parameters
                            .get("content")
                            .or_else(|| tool_req.parameters.get("text"))
                            .or_else(|| tool_req.parameters.get("data"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        tools.push(AvailableTool::FileWrite {
                            path: path.to_string(),
                            content,
                        });
                    }
                }
                "WebSearch" => {
                    if let Some(query) = tool_req.parameters.get("query").and_then(|v| v.as_str()) {
                        tools.push(AvailableTool::WebSearch {
                            query: query.to_string(),
                            max_uses: None,
                            allowed_domains: None,
                            blocked_domains: None,
                            user_location: None,
                        });
                    }
                }

                // Git operations
                "GitStatus" => {
                    let repository_path = tool_req
                        .parameters
                        .get("repository_path")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    tools.push(AvailableTool::GitStatus { repository_path });
                }
                "GitAdd" => {
                    let files = tool_req
                        .parameters
                        .get("files")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str())
                                .map(|s| s.to_string())
                                .collect()
                        })
                        .unwrap_or_else(|| vec![".".to_string()]);

                    let repository_path = tool_req
                        .parameters
                        .get("repository_path")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    tools.push(AvailableTool::GitAdd {
                        files,
                        repository_path,
                    });
                }
                "GitCommit" => {
                    if let Some(message) =
                        tool_req.parameters.get("message").and_then(|v| v.as_str())
                    {
                        let repository_path = tool_req
                            .parameters
                            .get("repository_path")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());

                        tools.push(AvailableTool::GitCommit {
                            message: message.to_string(),
                            repository_path,
                        });
                    }
                }

                // HTTP/API operations
                "HttpRequest" => {
                    if let Some(url) = tool_req.parameters.get("url").and_then(|v| v.as_str()) {
                        let method = tool_req
                            .parameters
                            .get("method")
                            .and_then(|v| v.as_str())
                            .and_then(|m| match m.to_uppercase().as_str() {
                                "GET" => Some(HttpMethod::GET),
                                "POST" => Some(HttpMethod::POST),
                                "PUT" => Some(HttpMethod::PUT),
                                "DELETE" => Some(HttpMethod::DELETE),
                                "PATCH" => Some(HttpMethod::PATCH),
                                _ => None,
                            })
                            .unwrap_or(HttpMethod::GET);

                        let headers = tool_req
                            .parameters
                            .get("headers")
                            .and_then(|v| v.as_object())
                            .map(|obj| {
                                obj.iter()
                                    .filter_map(|(k, v)| {
                                        v.as_str().map(|s| (k.clone(), s.to_string()))
                                    })
                                    .collect()
                            });

                        let body = tool_req
                            .parameters
                            .get("body")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());

                        tools.push(AvailableTool::HttpRequest {
                            method,
                            url: url.to_string(),
                            headers,
                            body,
                            timeout_seconds: None,
                        });
                    }
                }

                // Model configuration
                "SetModelParameter" => {
                    if let Some(param_str) = tool_req
                        .parameters
                        .get("parameter")
                        .and_then(|v| v.as_str())
                    {
                        let parameter = match param_str.to_lowercase().as_str() {
                            "temperature" => Some(ModelParameter::Temperature),
                            "max_tokens" | "maxtokens" => Some(ModelParameter::MaxTokens),
                            "top_p" | "topp" => Some(ModelParameter::TopP),
                            "top_k" | "topk" => Some(ModelParameter::TopK),
                            "repeat_penalty" | "repeatpenalty" => {
                                Some(ModelParameter::RepeatPenalty)
                            }
                            "system_prompt" | "systemprompt" => Some(ModelParameter::SystemPrompt),
                            "context_length" | "contextlength" => {
                                Some(ModelParameter::ContextLength)
                            }
                            _ => None,
                        };

                        if let (Some(parameter), Some(value)) =
                            (parameter, tool_req.parameters.get("value"))
                        {
                            tools.push(AvailableTool::SetModelParameter {
                                parameter,
                                value: value.clone(),
                            });
                        }
                    }
                }
                "GetModelParameter" => {
                    let parameter = tool_req
                        .parameters
                        .get("parameter")
                        .and_then(|v| v.as_str())
                        .and_then(|param_str| match param_str.to_lowercase().as_str() {
                            "temperature" => Some(ModelParameter::Temperature),
                            "max_tokens" => Some(ModelParameter::MaxTokens),
                            "top_p" => Some(ModelParameter::TopP),
                            "top_k" => Some(ModelParameter::TopK),
                            "repeat_penalty" => Some(ModelParameter::RepeatPenalty),
                            "system_prompt" => Some(ModelParameter::SystemPrompt),
                            "context_length" => Some(ModelParameter::ContextLength),
                            _ => None,
                        });

                    tools.push(AvailableTool::GetModelParameter { parameter });
                }
                "SwitchModel" => {
                    let model_name = tool_req
                        .parameters
                        .get("model_name")
                        .or_else(|| tool_req.parameters.get("model"))
                        .or_else(|| tool_req.parameters.get("name"))
                        .and_then(|v| v.as_str());
                    
                    if let Some(model_name) = model_name {
                        tools.push(AvailableTool::SwitchModel {
                            model_name: model_name.to_string(),
                        });
                    }
                }

                // System operations
                "SystemInfo" => {
                    tools.push(AvailableTool::SystemInfo);
                }
                "MemoryUsage" => {
                    tools.push(AvailableTool::MemoryUsage);
                }
                "DiskUsage" => {
                    let path = tool_req
                        .parameters
                        .get("path")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    tools.push(AvailableTool::DiskUsage { path });
                }
                "ProcessList" => {
                    let filter = tool_req
                        .parameters
                        .get("filter")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    tools.push(AvailableTool::ProcessList { filter });
                }

                // File watching
                "FileWatch" => {
                    // Try multiple parameter names for flexibility
                    let path = tool_req.parameters.get("path")
                        .or_else(|| tool_req.parameters.get("filename"))
                        .or_else(|| tool_req.parameters.get("file"))
                        .or_else(|| tool_req.parameters.get("filepath"))
                        .and_then(|v| v.as_str());
                    
                    if let Some(path) = path {
                        let duration_seconds = tool_req
                            .parameters
                            .get("duration_seconds")
                            .or_else(|| tool_req.parameters.get("duration"))
                            .or_else(|| tool_req.parameters.get("time"))
                            .or_else(|| tool_req.parameters.get("seconds"))
                            .and_then(|v| {
                                // Handle different formats: "60s", "60", 60, "1 minute", "2 minutes"
                                if let Some(s) = v.as_str() {
                                    let s = s.trim().to_lowercase();
                                    if s.ends_with(" minutes") || s.ends_with(" minute") {
                                        s.split_whitespace().next()
                                            .and_then(|num| num.parse::<u64>().ok())
                                            .map(|n| n * 60)
                                    } else if s.ends_with(" seconds") || s.ends_with(" second") {
                                        s.split_whitespace().next()
                                            .and_then(|num| num.parse::<u64>().ok())
                                    } else if s.ends_with('s') {
                                        s.trim_end_matches('s').parse::<u64>().ok()
                                    } else if s.ends_with("min") {
                                        s.trim_end_matches("min").parse::<u64>().ok().map(|n| n * 60)
                                    } else {
                                        s.parse::<u64>().ok()
                                    }
                                } else {
                                    v.as_u64()
                                }
                            });
                        tools.push(AvailableTool::FileWatch {
                            path: path.to_string(),
                            duration_seconds,
                        });
                    }
                }

                // File operations
                "FileSearch" => {
                    if let Some(pattern) = tool_req.parameters.get("pattern").and_then(|v| v.as_str()) {
                        let directory = tool_req.parameters.get("path").and_then(|v| v.as_str()).map(|s| s.to_string());
                        tools.push(AvailableTool::FileSearch {
                            pattern: pattern.to_string(),
                            directory,
                        });
                    }
                }
                "ContentSearch" => {
                    if let Some(pattern) = tool_req.parameters.get("pattern").and_then(|v| v.as_str()) {
                        let directory = tool_req.parameters.get("path").and_then(|v| v.as_str()).map(|s| s.to_string());
                        tools.push(AvailableTool::ContentSearch {
                            pattern: pattern.to_string(),
                            directory,
                        });
                    }
                }
                "ListDirectory" => {
                    let path = tool_req.parameters.get("path").and_then(|v| v.as_str()).unwrap_or(".");
                    tools.push(AvailableTool::ListDirectory {
                        path: path.to_string(),
                    });
                }
                "ExecuteCommand" => {
                    if let Some(command) = tool_req.parameters.get("command").and_then(|v| v.as_str()) {
                        tools.push(AvailableTool::ExecuteCommand {
                            command: command.to_string(),
                        });
                    }
                }
                
                // Project operations
                "CreateProject" => {
                    if let Some(name) = tool_req.parameters.get("name").and_then(|v| v.as_str()) {
                        let project_type = tool_req.parameters.get("template").and_then(|v| v.as_str()).unwrap_or("basic");
                        let path = tool_req.parameters.get("path").and_then(|v| v.as_str()).map(|s| s.to_string());
                        tools.push(AvailableTool::CreateProject {
                            name: name.to_string(),
                            project_type: project_type.to_string(),
                            path,
                        });
                    }
                }
                
                // Package management
                "CargoOperation" => {
                    if let Some(action) = tool_req.parameters.get("action").and_then(|v| v.as_str()) {
                        let operation = match action.to_lowercase().as_str() {
                            "build" => CargoOperation::Build,
                            "run" => CargoOperation::Run,
                            "test" => CargoOperation::Test,
                            "check" => CargoOperation::Check,
                            "install" => CargoOperation::Install,
                            _ => CargoOperation::Build, // default
                        };
                        let package = tool_req.parameters.get("package").and_then(|v| v.as_str()).map(|s| s.to_string());
                        tools.push(AvailableTool::CargoOperation {
                            operation,
                            package,
                            features: None,
                        });
                    }
                }
                "NpmOperation" => {
                    if let Some(action) = tool_req.parameters.get("action").and_then(|v| v.as_str()) {
                        let operation = match action.to_lowercase().as_str() {
                            "install" => NpmOperation::Install,
                            "uninstall" => NpmOperation::Uninstall,
                            "update" => NpmOperation::Update,
                            "audit" => NpmOperation::Audit,
                            script => NpmOperation::Run { script: script.to_string() },
                        };
                        let package = tool_req.parameters.get("package").and_then(|v| v.as_str()).map(|s| s.to_string());
                        tools.push(AvailableTool::NpmOperation {
                            operation,
                            package,
                            dev: false,
                        });
                    }
                }
                "PipOperation" => {
                    if let Some(action) = tool_req.parameters.get("action").and_then(|v| v.as_str()) {
                        let operation = match action.to_lowercase().as_str() {
                            "install" => PipOperation::Install,
                            "uninstall" => PipOperation::Uninstall,
                            "list" => PipOperation::List,
                            "freeze" => PipOperation::Freeze,
                            "show" => PipOperation::Show,
                            _ => PipOperation::List, // default
                        };
                        let package = tool_req.parameters.get("package").and_then(|v| v.as_str()).map(|s| s.to_string());
                        tools.push(AvailableTool::PipOperation {
                            operation,
                            package,
                            requirements_file: None,
                        });
                    }
                }
                
                // Docker operations
                "DockerList" => {
                    tools.push(AvailableTool::DockerList {
                        resource_type: DockerResourceType::Containers,
                    });
                }
                "DockerRun" => {
                    if let Some(image) = tool_req.parameters.get("image").and_then(|v| v.as_str()) {
                        let command = tool_req.parameters.get("command").and_then(|v| v.as_str()).map(|s| s.to_string());
                        tools.push(AvailableTool::DockerRun {
                            image: image.to_string(),
                            command,
                            ports: None,
                            volumes: None,
                            environment: None,
                        });
                    }
                }
                "DockerStop" => {
                    if let Some(container) = tool_req.parameters.get("container").and_then(|v| v.as_str()) {
                        tools.push(AvailableTool::DockerStop {
                            container: container.to_string(),
                        });
                    }
                }
                "DockerLogs" => {
                    if let Some(container) = tool_req.parameters.get("container").and_then(|v| v.as_str()) {
                        tools.push(AvailableTool::DockerLogs {
                            container: container.to_string(),
                            follow: false,
                            tail: None,
                        });
                    }
                }
                
                // Text processing
                "JsonFormat" => {
                    if let Some(input) = tool_req.parameters.get("json").and_then(|v| v.as_str()) {
                        tools.push(AvailableTool::JsonFormat {
                            input: input.to_string(),
                        });
                    }
                }
                "JsonQuery" => {
                    if let Some(input) = tool_req.parameters.get("json").and_then(|v| v.as_str()) {
                        if let Some(query) = tool_req.parameters.get("query").and_then(|v| v.as_str()) {
                            tools.push(AvailableTool::JsonQuery {
                                input: input.to_string(),
                                query: query.to_string(),
                            });
                        }
                    }
                }
                "CsvParse" => {
                    if let Some(input) = tool_req.parameters.get("csv").and_then(|v| v.as_str()) {
                        tools.push(AvailableTool::CsvParse {
                            input: input.to_string(),
                            delimiter: None,
                        });
                    }
                }
                "RegexMatch" => {
                    if let Some(text) = tool_req.parameters.get("text").and_then(|v| v.as_str()) {
                        if let Some(pattern) = tool_req.parameters.get("pattern").and_then(|v| v.as_str()) {
                            tools.push(AvailableTool::RegexMatch {
                                pattern: pattern.to_string(),
                                text: text.to_string(),
                                flags: None,
                            });
                        }
                    }
                }
                "TextTransform" => {
                    if let Some(input) = tool_req.parameters.get("text").and_then(|v| v.as_str()) {
                        if let Some(operation_str) = tool_req.parameters.get("operation").and_then(|v| v.as_str()) {
                            let operation = match operation_str.to_lowercase().as_str() {
                                "uppercase" => TextOperation::ToUpperCase,
                                "lowercase" => TextOperation::ToLowerCase,
                                "trim" => TextOperation::Trim,
                                _ => TextOperation::ToLowerCase, // default
                            };
                            tools.push(AvailableTool::TextTransform {
                                input: input.to_string(),
                                operation,
                            });
                        }
                    }
                }
                
                // Configuration operations
                "ExportConversation" => {
                    if let Some(path) = tool_req.parameters.get("filename").and_then(|v| v.as_str()) {
                        tools.push(AvailableTool::ExportConversation {
                            format: ExportFormat::Json,
                            path: path.to_string(),
                        });
                    }
                }
                "ImportConversation" => {
                    if let Some(path) = tool_req.parameters.get("filename").and_then(|v| v.as_str()) {
                        tools.push(AvailableTool::ImportConversation {
                            path: path.to_string(),
                        });
                    }
                }
                "ClearHistory" => {
                    tools.push(AvailableTool::ClearHistory);
                }
                
                // Task scheduling
                "ScheduleTask" => {
                    if let Some(command) = tool_req.parameters.get("command").and_then(|v| v.as_str()) {
                        if let Some(schedule) = tool_req.parameters.get("schedule").and_then(|v| v.as_str()) {
                            let name = tool_req.parameters.get("name").and_then(|v| v.as_str()).map(|s| s.to_string());
                            tools.push(AvailableTool::ScheduleTask {
                                command: command.to_string(),
                                schedule: schedule.to_string(),
                                name,
                            });
                        }
                    }
                }
                "ListScheduledTasks" => {
                    tools.push(AvailableTool::ListScheduledTasks);
                }
                "CancelScheduledTask" => {
                    if let Some(name) = tool_req.parameters.get("name").and_then(|v| v.as_str()) {
                        tools.push(AvailableTool::CancelScheduledTask {
                            name: name.to_string(),
                        });
                    }
                }
                
                // Add more tool conversions here...
                _ => {
                    println!(
                        "  {} Unknown tool type: {}",
                        "⚠".yellow(),
                        tool_req.tool_type
                    );
                }
            }
        }

        tools
    }

    fn enhanced_fallback_parse(&self, input: &str) -> Vec<AvailableTool> {
        // Enhanced fallback parsing when LLM fails
        let input_lower = input.to_lowercase();
        let mut tools = Vec::new();
        
        // File operations
        if input_lower.contains("read") || input_lower.contains("show") || input_lower.contains("cat") {
            if let Some(filename) = self.extract_filename(input) {
                tools.push(AvailableTool::FileRead { path: filename });
            }
        }
        
        // Directory listing
        if input_lower.contains("list") || input_lower.contains("ls") || input_lower.contains("dir") {
            let path = self.extract_path(input).unwrap_or_else(|| ".".to_string());
            tools.push(AvailableTool::ListDirectory { path });
        }
        
        // File search
        if input_lower.contains("find") || input_lower.contains("search") {
            if let Some(pattern) = self.extract_search_pattern(input) {
                tools.push(AvailableTool::FileSearch { 
                    pattern, 
                    directory: None 
                });
            }
        }
        
        // Git operations
        if input_lower.contains("git status") {
            tools.push(AvailableTool::GitStatus { repository_path: None });
        }
        
        // System info
        if input_lower.contains("system") && input_lower.contains("info") {
            tools.push(AvailableTool::SystemInfo);
        }
        
        // Process list
        if input_lower.contains("process") || input_lower.contains("ps") {
            tools.push(AvailableTool::ProcessList { filter: None });
        }
        
        tools
    }
    
    fn extract_filename(&self, input: &str) -> Option<String> {
        // Common file extensions
        let extensions = [".rs", ".toml", ".json", ".md", ".txt", ".py", ".js", ".ts", ".go", ".c", ".cpp", ".h"];
        
        for ext in extensions {
            if let Some(start) = input.find(ext) {
                // Look backwards to find the start of the filename
                let before = &input[..start];
                if let Some(space_pos) = before.rfind(' ') {
                    return Some(input[space_pos + 1..start + ext.len()].to_string());
                } else {
                    return Some(input[..start + ext.len()].to_string());
                }
            }
        }
        
        None
    }
    
    fn extract_path(&self, input: &str) -> Option<String> {
        // Look for path patterns like "src/", "./", "../", etc.
        let words: Vec<&str> = input.split_whitespace().collect();
        for word in words {
            if word.contains('/') && !word.starts_with("http") {
                return Some(word.to_string());
            }
        }
        None
    }
    
    fn extract_search_pattern(&self, input: &str) -> Option<String> {
        // Look for patterns like "find *.rs", "search for TODO", etc.
        let words: Vec<&str> = input.split_whitespace().collect();
        for (i, word) in words.iter().enumerate() {
            if *word == "find" || *word == "search" {
                if i + 1 < words.len() {
                    let next_word = words[i + 1];
                    if next_word != "for" && next_word != "in" {
                        return Some(next_word.to_string());
                    } else if i + 2 < words.len() {
                        return Some(words[i + 2].to_string());
                    }
                }
            }
        }
        None
    }

    fn simple_fallback_parse(&self, _input: &str) -> Vec<AvailableTool> {
        // Remove all fallback parsing - let LLM handle everything
        vec![]
    }

    // Keep this for backward compatibility
    pub fn parse_request(&self, input: &str) -> Vec<AvailableTool> {
        self.enhanced_fallback_parse(input)
    }

    pub fn suggest_clarification(&self, input: &str) -> Option<String> {
        let lower_input = input.to_lowercase();

        if lower_input.contains("model")
            || lower_input.contains("temperature")
            || lower_input.contains("parameter")
        {
            Some("I can help with model configuration. Try: 'set temperature to 0.8', 'show model config', or 'switch to llama2'".to_string())
        } else if lower_input.contains("git") {
            Some("I can help with git operations. Try: 'git status', 'git add main.rs', 'commit with message', or 'push changes'".to_string())
        } else if lower_input.contains("api") || lower_input.contains("http") {
            Some("I can help with API calls. Try: 'GET request to api.example.com', 'scrape https://example.com', or 'query GraphQL endpoint'".to_string())
        } else if lower_input.contains("docker") {
            Some("I can help with Docker. Try: 'list containers', 'run nginx container', or 'show docker logs for myapp'".to_string())
        } else if lower_input.contains("system") {
            Some("I can help with system operations. Try: 'system info', 'memory usage', 'disk space', or 'list processes'".to_string())
        } else if lower_input.contains("file") {
            Some("I can help with file operations. Try: 'read Cargo.toml', 'search for *.rs files', or 'create file.txt with content Hello'".to_string())
        } else if lower_input.contains("search") {
            Some("I can search the web or files. Try: 'search for rust programming' or 'search for \"function main\" in src/'".to_string())
        } else if lower_input.contains("project") {
            Some("I can create projects. Try: 'create a rust project called my-app' or 'make a python project named calculator'".to_string())
        } else {
            None
        }
    }
}
