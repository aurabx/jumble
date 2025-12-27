use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::env;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// An MCP server that provides queryable, on-demand project context to LLMs
#[derive(Parser, Debug)]
#[command(name = "jumble", version, about)]
struct Args {
    /// Root directory to scan for .jumble/project.toml files
    #[arg(long, env = "JUMBLE_ROOT")]
    root: Option<PathBuf>,
}

// ============================================================================
// Project Configuration Types
// ============================================================================

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ProjectConfig {
    project: ProjectInfo,
    #[serde(default)]
    commands: HashMap<String, String>,
    #[serde(default)]
    entry_points: HashMap<String, String>,
    #[serde(default)]
    dependencies: Dependencies,
    #[serde(default)]
    related_projects: RelatedProjects,
    #[serde(default)]
    api: Option<ApiInfo>,
    #[serde(default)]
    concepts: HashMap<String, Concept>,
}

/// Discovered prompts for a project (from .jumble/prompts/*.md)
#[derive(Debug, Clone, Default)]
struct ProjectPrompts {
    prompts: HashMap<String, PathBuf>,
}

/// Conventions and gotchas for a project (from .jumble/conventions.toml)
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
struct ProjectConventions {
    #[serde(default)]
    conventions: HashMap<String, String>,
    #[serde(default)]
    gotchas: HashMap<String, String>,
}

/// Documentation index for a project (from .jumble/docs.toml)
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
struct ProjectDocs {
    #[serde(default)]
    docs: HashMap<String, DocEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct DocEntry {
    path: String,
    summary: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ProjectInfo {
    name: String,
    description: String,
    #[serde(default)]
    language: Option<String>,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    repository: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
struct Dependencies {
    #[serde(default)]
    internal: Vec<String>,
    #[serde(default)]
    external: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
struct RelatedProjects {
    #[serde(default)]
    upstream: Vec<String>,
    #[serde(default)]
    downstream: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ApiInfo {
    #[serde(default)]
    openapi: Option<String>,
    #[serde(default)]
    base_url: Option<String>,
    #[serde(default)]
    endpoints: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Concept {
    files: Vec<String>,
    summary: String,
}

// ============================================================================
// Workspace Configuration (from .jumble/workspace.toml at root)
// ============================================================================

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
struct WorkspaceConfig {
    #[serde(default)]
    workspace: WorkspaceInfo,
    #[serde(default)]
    conventions: HashMap<String, String>,
    #[serde(default)]
    gotchas: HashMap<String, String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
struct WorkspaceInfo {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    description: Option<String>,
}

// ============================================================================
// MCP Protocol Types
// ============================================================================

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

// ============================================================================
// Server State
// ============================================================================

struct Server {
    root: PathBuf,
    workspace: Option<WorkspaceConfig>,
    projects: HashMap<String, (PathBuf, ProjectConfig, ProjectPrompts, ProjectConventions, ProjectDocs)>,
}

impl Server {
    fn new(root: PathBuf) -> Result<Self> {
        let workspace = Self::load_workspace_static(&root);
        let mut server = Server {
            root,
            workspace,
            projects: HashMap::new(),
        };
        server.discover_projects()?;
        Ok(server)
    }

    fn load_workspace_static(root: &Path) -> Option<WorkspaceConfig> {
        let workspace_path = root.join(".jumble/workspace.toml");
        if workspace_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&workspace_path) {
                if let Ok(config) = toml::from_str(&content) {
                    return Some(config);
                }
            }
        }
        None
    }

    fn discover_projects(&mut self) -> Result<()> {
        for entry in WalkDir::new(&self.root)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.ends_with(".jumble/project.toml") {
                if let Ok(config) = self.load_project(path) {
                    let project_dir = path
                        .parent()
                        .and_then(|p| p.parent())
                        .unwrap_or(path)
                        .to_path_buf();
                    
                    // Discover prompts, conventions, and docs
                    let prompts = self.discover_prompts(path.parent().unwrap());
                    let conventions = self.load_conventions(path.parent().unwrap());
                    let docs = self.load_docs(path.parent().unwrap());
                    
                    self.projects
                        .insert(config.project.name.clone(), (project_dir, config, prompts, conventions, docs));
                }
            }
        }
        Ok(())
    }

    fn discover_prompts(&self, jumble_dir: &Path) -> ProjectPrompts {
        let mut prompts = ProjectPrompts::default();
        let prompts_dir = jumble_dir.join("prompts");
        
        if prompts_dir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&prompts_dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.extension().map(|e| e == "md").unwrap_or(false) {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            prompts.prompts.insert(stem.to_string(), path);
                        }
                    }
                }
            }
        }
        
        prompts
    }

    fn load_conventions(&self, jumble_dir: &Path) -> ProjectConventions {
        let conventions_path = jumble_dir.join("conventions.toml");
        
        if conventions_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&conventions_path) {
                if let Ok(conventions) = toml::from_str(&content) {
                    return conventions;
                }
            }
        }
        
        ProjectConventions::default()
    }

    fn load_docs(&self, jumble_dir: &Path) -> ProjectDocs {
        let docs_path = jumble_dir.join("docs.toml");
        
        if docs_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&docs_path) {
                if let Ok(docs) = toml::from_str(&content) {
                    return docs;
                }
            }
        }
        
        ProjectDocs::default()
    }

    fn load_project(&self, path: &Path) -> Result<ProjectConfig> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let config: ProjectConfig =
            toml::from_str(&content).with_context(|| format!("Failed to parse {}", path.display()))?;
        Ok(config)
    }

    fn handle_request(&mut self, request: JsonRpcRequest) -> JsonRpcResponse {
        let result = match request.method.as_str() {
            "initialize" => self.handle_initialize(&request.params),
            "initialized" => Ok(json!({})),
            "tools/list" => self.handle_tools_list(),
            "tools/call" => self.handle_tools_call(&request.params),
            _ => Err(JsonRpcError {
                code: -32601,
                message: format!("Method not found: {}", request.method),
                data: None,
            }),
        };

        match result {
            Ok(value) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(value),
                error: None,
            },
            Err(error) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(error),
            },
        }
    }

    fn handle_initialize(&self, _params: &Value) -> Result<Value, JsonRpcError> {
        Ok(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "jumble",
                "version": env!("CARGO_PKG_VERSION")
            }
        }))
    }

    fn handle_tools_list(&self) -> Result<Value, JsonRpcError> {
        Ok(json!({
            "tools": [
                {
                    "name": "list_projects",
                    "description": "Lists all projects with their descriptions. Use this to discover what projects exist in the workspace.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {},
                        "required": []
                    }
                },
                {
                    "name": "get_project_info",
                    "description": "Returns metadata about a specific project including description, language, version, entry points, and dependencies.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "project": {
                                "type": "string",
                                "description": "The project name"
                            },
                            "field": {
                                "type": "string",
                                "description": "Optional specific field to retrieve: 'commands', 'entry_points', 'dependencies', 'api', 'related_projects'",
                                "enum": ["commands", "entry_points", "dependencies", "api", "related_projects"]
                            }
                        },
                        "required": ["project"]
                    }
                },
                {
                    "name": "get_commands",
                    "description": "Returns executable commands for a project (build, test, lint, run, dev, etc.)",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "project": {
                                "type": "string",
                                "description": "The project name"
                            },
                            "command_type": {
                                "type": "string",
                                "description": "Optional specific command type: 'build', 'test', 'lint', 'run', 'dev'"
                            }
                        },
                        "required": ["project"]
                    }
                },
                {
                    "name": "get_architecture",
                    "description": "Returns architectural info for a specific concept/area of a project, including relevant files and a summary.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "project": {
                                "type": "string",
                                "description": "The project name"
                            },
                            "concept": {
                                "type": "string",
                                "description": "The architectural concept to look up (e.g., 'authentication', 'routing', 'database')"
                            }
                        },
                        "required": ["project", "concept"]
                    }
                },
                {
                    "name": "get_related_files",
                    "description": "Finds files related to a concept or feature by searching through all defined concepts.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "project": {
                                "type": "string",
                                "description": "The project name"
                            },
                            "query": {
                                "type": "string",
                                "description": "Search query to match against concept names and summaries"
                            }
                        },
                        "required": ["project", "query"]
                    }
                },
                {
                    "name": "list_prompts",
                    "description": "Lists available task-specific prompts for a project. Prompts provide focused context for specific tasks like adding endpoints, debugging, etc.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "project": {
                                "type": "string",
                                "description": "The project name"
                            }
                        },
                        "required": ["project"]
                    }
                },
                {
                    "name": "get_prompt",
                    "description": "Retrieves a task-specific prompt containing focused context and instructions for a particular task.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "project": {
                                "type": "string",
                                "description": "The project name"
                            },
                            "topic": {
                                "type": "string",
                                "description": "The prompt topic (e.g., 'add-endpoint', 'debug-auth')"
                            }
                        },
                        "required": ["project", "topic"]
                    }
                },
                {
                    "name": "get_conventions",
                    "description": "Returns project-specific coding conventions and gotchas. Conventions are architectural patterns and standards; gotchas are common mistakes to avoid.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "project": {
                                "type": "string",
                                "description": "The project name"
                            },
                            "category": {
                                "type": "string",
                                "description": "Optional: 'conventions' or 'gotchas' to filter results",
                                "enum": ["conventions", "gotchas"]
                            }
                        },
                        "required": ["project"]
                    }
                },
                {
                    "name": "get_docs",
                    "description": "Returns a documentation index for a project, listing available docs with summaries. Optionally retrieves the path to a specific doc.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "project": {
                                "type": "string",
                                "description": "The project name"
                            },
                            "topic": {
                                "type": "string",
                                "description": "Optional: specific doc topic to get the path for"
                            }
                        },
                        "required": ["project"]
                    }
                },
                {
                    "name": "get_workspace_overview",
                    "description": "Returns a high-level overview of the entire workspace: workspace info, all projects with descriptions, and their dependency relationships. Call this first to understand the workspace structure.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {},
                        "required": []
                    }
                },
                {
                    "name": "get_workspace_conventions",
                    "description": "Returns workspace-level conventions and gotchas that apply across all projects in the workspace.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "category": {
                                "type": "string",
                                "description": "Optional: 'conventions' or 'gotchas' to filter results",
                                "enum": ["conventions", "gotchas"]
                            }
                        },
                        "required": []
                    }
                }
            ]
        }))
    }

    fn handle_tools_call(&mut self, params: &Value) -> Result<Value, JsonRpcError> {
        let name = params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| JsonRpcError {
                code: -32602,
                message: "Missing 'name' parameter".to_string(),
                data: None,
            })?;

        let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

        let result = match name {
            "list_projects" => self.tool_list_projects(),
            "get_project_info" => self.tool_get_project_info(&arguments),
            "get_commands" => self.tool_get_commands(&arguments),
            "get_architecture" => self.tool_get_architecture(&arguments),
            "get_related_files" => self.tool_get_related_files(&arguments),
            "list_prompts" => self.tool_list_prompts(&arguments),
            "get_prompt" => self.tool_get_prompt(&arguments),
            "get_conventions" => self.tool_get_conventions(&arguments),
            "get_docs" => self.tool_get_docs(&arguments),
            "get_workspace_overview" => self.tool_get_workspace_overview(),
            "get_workspace_conventions" => self.tool_get_workspace_conventions(&arguments),
            _ => Err(format!("Unknown tool: {}", name)),
        };

        match result {
            Ok(content) => Ok(json!({
                "content": [{
                    "type": "text",
                    "text": content
                }]
            })),
            Err(msg) => Ok(json!({
                "content": [{
                    "type": "text",
                    "text": format!("Error: {}", msg)
                }],
                "isError": true
            })),
        }
    }

    // ========================================================================
    // Tool Implementations
    // ========================================================================

    fn tool_list_projects(&self) -> Result<String, String> {
        if self.projects.is_empty() {
            return Ok("No projects found. Make sure .jumble/project.toml files exist in your workspace.".to_string());
        }

        let mut output = String::new();
        for (name, (path, config, _prompts, _conventions, _docs)) in &self.projects {
            let lang = config
                .project
                .language
                .as_deref()
                .unwrap_or("unknown");
            output.push_str(&format!(
                "- **{}** ({}): {}\n  Path: {}\n",
                name,
                lang,
                config.project.description,
                path.display()
            ));
        }
        Ok(output)
    }

    fn tool_get_project_info(&self, args: &Value) -> Result<String, String> {
        let project_name = args
            .get("project")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'project' argument")?;

        let (path, config, _prompts, _conventions, _docs) = self
            .projects
            .get(project_name)
            .ok_or_else(|| format!("Project '{}' not found", project_name))?;

        let field = args.get("field").and_then(|v| v.as_str());

        match field {
            Some("commands") => Ok(format_commands(&config.commands)),
            Some("entry_points") => Ok(format_entry_points(&config.entry_points)),
            Some("dependencies") => Ok(format_dependencies(&config.dependencies)),
            Some("api") => Ok(format_api(&config.api)),
            Some("related_projects") => Ok(format_related_projects(&config.related_projects)),
            Some(f) => Err(format!("Unknown field: {}", f)),
            None => {
                let mut output = format!("# {}\n\n", config.project.name);
                output.push_str(&format!("**Description:** {}\n", config.project.description));
                if let Some(lang) = &config.project.language {
                    output.push_str(&format!("**Language:** {}\n", lang));
                }
                if let Some(version) = &config.project.version {
                    output.push_str(&format!("**Version:** {}\n", version));
                }
                if let Some(repo) = &config.project.repository {
                    output.push_str(&format!("**Repository:** {}\n", repo));
                }
                output.push_str(&format!("**Path:** {}\n", path.display()));

                if !config.entry_points.is_empty() {
                    output.push_str("\n## Entry Points\n");
                    output.push_str(&format_entry_points(&config.entry_points));
                }

                if !config.concepts.is_empty() {
                    output.push_str("\n## Concepts\n");
                    for (name, concept) in &config.concepts {
                        output.push_str(&format!("- **{}**: {}\n", name, concept.summary));
                    }
                }

                Ok(output)
            }
        }
    }

    fn tool_get_commands(&self, args: &Value) -> Result<String, String> {
        let project_name = args
            .get("project")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'project' argument")?;

        let (_, config, _, _, _) = self
            .projects
            .get(project_name)
            .ok_or_else(|| format!("Project '{}' not found", project_name))?;

        let command_type = args.get("command_type").and_then(|v| v.as_str());

        match command_type {
            Some(cmd_type) => {
                config
                    .commands
                    .get(cmd_type)
                    .map(|cmd| format!("{}: {}", cmd_type, cmd))
                    .ok_or_else(|| format!("Command '{}' not found for project '{}'", cmd_type, project_name))
            }
            None => Ok(format_commands(&config.commands)),
        }
    }

    fn tool_get_architecture(&self, args: &Value) -> Result<String, String> {
        let project_name = args
            .get("project")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'project' argument")?;

        let concept_name = args
            .get("concept")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'concept' argument")?;

        let (path, config, _, _, _) = self
            .projects
            .get(project_name)
            .ok_or_else(|| format!("Project '{}' not found", project_name))?;

        // Try exact match first
        if let Some(concept) = config.concepts.get(concept_name) {
            return Ok(format_concept(path, concept_name, concept));
        }

        // Try case-insensitive match
        let concept_lower = concept_name.to_lowercase();
        for (name, concept) in &config.concepts {
            if name.to_lowercase() == concept_lower {
                return Ok(format_concept(path, name, concept));
            }
        }

        // Try partial match
        for (name, concept) in &config.concepts {
            if name.to_lowercase().contains(&concept_lower)
                || concept.summary.to_lowercase().contains(&concept_lower)
            {
                return Ok(format_concept(path, name, concept));
            }
        }

        // List available concepts
        let available: Vec<&str> = config.concepts.keys().map(|s| s.as_str()).collect();
        Err(format!(
            "Concept '{}' not found. Available concepts: {}",
            concept_name,
            available.join(", ")
        ))
    }

    fn tool_get_related_files(&self, args: &Value) -> Result<String, String> {
        let project_name = args
            .get("project")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'project' argument")?;

        let query = args
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'query' argument")?;

        let (path, config, _, _, _) = self
            .projects
            .get(project_name)
            .ok_or_else(|| format!("Project '{}' not found", project_name))?;

        let query_lower = query.to_lowercase();
        let mut matched_files: Vec<(String, &str, &Concept)> = Vec::new();

        for (name, concept) in &config.concepts {
            if name.to_lowercase().contains(&query_lower)
                || concept.summary.to_lowercase().contains(&query_lower)
            {
                matched_files.push((name.clone(), name.as_str(), concept));
            }
        }

        if matched_files.is_empty() {
            return Err(format!("No concepts matching '{}' found", query));
        }

        let mut output = format!("Files related to '{}': \n\n", query);
        for (_, name, concept) in &matched_files {
            output.push_str(&format!("## {}\n{}\n\nFiles:\n", name, concept.summary));
            for file in &concept.files {
                output.push_str(&format!("- {}/{}\n", path.display(), file));
            }
            output.push('\n');
        }

        Ok(output)
    }

    fn tool_list_prompts(&self, args: &Value) -> Result<String, String> {
        let project_name = args
            .get("project")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'project' argument")?;

        let (_, _, prompts, _, _) = self
            .projects
            .get(project_name)
            .ok_or_else(|| format!("Project '{}' not found", project_name))?;

        if prompts.prompts.is_empty() {
            return Ok(format!(
                "No prompts found for '{}'. Create .jumble/prompts/*.md files to add task-specific context.",
                project_name
            ));
        }

        let mut output = format!("Available prompts for '{}':\n\n", project_name);
        for name in prompts.prompts.keys() {
            output.push_str(&format!("- {}\n", name));
        }
        output.push_str("\nUse get_prompt(project, topic) to retrieve a specific prompt.");
        Ok(output)
    }

    fn tool_get_prompt(&self, args: &Value) -> Result<String, String> {
        let project_name = args
            .get("project")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'project' argument")?;

        let topic = args
            .get("topic")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'topic' argument")?;

        let (_, _, prompts, _, _) = self
            .projects
            .get(project_name)
            .ok_or_else(|| format!("Project '{}' not found", project_name))?;

        let prompt_path = prompts
            .prompts
            .get(topic)
            .ok_or_else(|| {
                let available: Vec<&str> = prompts.prompts.keys().map(|s| s.as_str()).collect();
                if available.is_empty() {
                    format!("No prompts found for '{}'", project_name)
                } else {
                    format!(
                        "Prompt '{}' not found. Available: {}",
                        topic,
                        available.join(", ")
                    )
                }
            })?;

        std::fs::read_to_string(prompt_path)
            .map_err(|e| format!("Failed to read prompt: {}", e))
    }

    fn tool_get_conventions(&self, args: &Value) -> Result<String, String> {
        let project_name = args
            .get("project")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'project' argument")?;

        let category = args.get("category").and_then(|v| v.as_str());

        let (_, _, _, conventions, _) = self
            .projects
            .get(project_name)
            .ok_or_else(|| format!("Project '{}' not found", project_name))?;

        let has_conventions = !conventions.conventions.is_empty();
        let has_gotchas = !conventions.gotchas.is_empty();

        if !has_conventions && !has_gotchas {
            return Ok(format!(
                "No conventions found for '{}'. Create .jumble/conventions.toml to add project-specific conventions and gotchas.",
                project_name
            ));
        }

        let mut output = String::new();

        match category {
            Some("conventions") => {
                if !has_conventions {
                    return Ok("No conventions defined.".to_string());
                }
                output.push_str(&format!("# Conventions for '{}'\n\n", project_name));
                for (name, desc) in &conventions.conventions {
                    output.push_str(&format!("## {}\n{}\n\n", name, desc));
                }
            }
            Some("gotchas") => {
                if !has_gotchas {
                    return Ok("No gotchas defined.".to_string());
                }
                output.push_str(&format!("# Gotchas for '{}'\n\n", project_name));
                for (name, desc) in &conventions.gotchas {
                    output.push_str(&format!("## {}\n{}\n\n", name, desc));
                }
            }
            None => {
                if has_conventions {
                    output.push_str(&format!("# Conventions for '{}'\n\n", project_name));
                    for (name, desc) in &conventions.conventions {
                        output.push_str(&format!("## {}\n{}\n\n", name, desc));
                    }
                }
                if has_gotchas {
                    output.push_str(&format!("# Gotchas for '{}'\n\n", project_name));
                    for (name, desc) in &conventions.gotchas {
                        output.push_str(&format!("## {}\n{}\n\n", name, desc));
                    }
                }
            }
            Some(c) => return Err(format!("Unknown category '{}'. Use 'conventions' or 'gotchas'.", c)),
        }

        Ok(output)
    }

    fn tool_get_docs(&self, args: &Value) -> Result<String, String> {
        let project_name = args
            .get("project")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'project' argument")?;

        let topic = args.get("topic").and_then(|v| v.as_str());

        let (path, _, _, _, docs) = self
            .projects
            .get(project_name)
            .ok_or_else(|| format!("Project '{}' not found", project_name))?;

        if docs.docs.is_empty() {
            return Ok(format!(
                "No documentation index found for '{}'. Create .jumble/docs.toml to index project documentation.",
                project_name
            ));
        }

        match topic {
            Some(t) => {
                // Return path to specific doc
                let doc = docs.docs.get(t).ok_or_else(|| {
                    let available: Vec<&str> = docs.docs.keys().map(|s| s.as_str()).collect();
                    format!(
                        "Doc '{}' not found. Available: {}",
                        t,
                        available.join(", ")
                    )
                })?;
                let full_path = path.join(&doc.path);
                Ok(format!(
                    "## {}\n**Summary:** {}\n**Path:** {}",
                    t, doc.summary, full_path.display()
                ))
            }
            None => {
                // List all docs with summaries
                let mut output = format!("# Documentation for '{}'\n\n", project_name);
                for (name, doc) in &docs.docs {
                    output.push_str(&format!("- **{}**: {}\n", name, doc.summary));
                }
                output.push_str("\nUse get_docs(project, topic) to get the path to a specific doc.");
                Ok(output)
            }
        }
    }

    fn tool_get_workspace_overview(&self) -> Result<String, String> {
        let mut output = String::new();

        // Workspace info
        if let Some(ws) = &self.workspace {
            if let Some(name) = &ws.workspace.name {
                output.push_str(&format!("# {}\n\n", name));
            } else {
                output.push_str("# Workspace Overview\n\n");
            }
            if let Some(desc) = &ws.workspace.description {
                output.push_str(&format!("{}\n\n", desc));
            }
        } else {
            output.push_str("# Workspace Overview\n\n");
        }

        output.push_str(&format!("**Root:** {}\n\n", self.root.display()));

        // Projects list
        if self.projects.is_empty() {
            output.push_str("No projects found.\n");
            return Ok(output);
        }

        output.push_str("## Projects\n\n");
        
        // Collect and sort projects for consistent output
        let mut project_names: Vec<&String> = self.projects.keys().collect();
        project_names.sort();

        for name in &project_names {
            let (_, config, _, _, _) = self.projects.get(*name).unwrap();
            let lang = config.project.language.as_deref().unwrap_or("unknown");
            output.push_str(&format!(
                "- **{}** ({}): {}\n",
                name, lang, config.project.description
            ));
        }

        // Dependency graph
        output.push_str("\n## Dependencies\n\n");
        let mut has_deps = false;
        
        for name in &project_names {
            let (_, config, _, _, _) = self.projects.get(*name).unwrap();
            let upstream = &config.related_projects.upstream;
            let downstream = &config.related_projects.downstream;
            
            if !upstream.is_empty() || !downstream.is_empty() {
                has_deps = true;
                output.push_str(&format!("**{}**:\n", name));
                if !upstream.is_empty() {
                    output.push_str(&format!("  ← depends on: {}\n", upstream.join(", ")));
                }
                if !downstream.is_empty() {
                    output.push_str(&format!("  → used by: {}\n", downstream.join(", ")));
                }
            }
        }
        
        if !has_deps {
            output.push_str("No cross-project dependencies defined.\n");
        }

        // Note about workspace conventions
        if self.workspace.is_some() {
            output.push_str("\n*Use get_workspace_conventions() for workspace-wide coding standards.*");
        }

        Ok(output)
    }

    fn tool_get_workspace_conventions(&self, args: &Value) -> Result<String, String> {
        let ws = self.workspace.as_ref().ok_or(
            "No workspace.toml found. Create .jumble/workspace.toml at the workspace root to define workspace-level conventions."
        )?;

        let category = args.get("category").and_then(|v| v.as_str());

        let has_conventions = !ws.conventions.is_empty();
        let has_gotchas = !ws.gotchas.is_empty();

        if !has_conventions && !has_gotchas {
            return Ok("Workspace config exists but no conventions or gotchas defined.".to_string());
        }

        let mut output = String::new();
        let ws_name = ws.workspace.name.as_deref().unwrap_or("Workspace");

        match category {
            Some("conventions") => {
                if !has_conventions {
                    return Ok("No workspace conventions defined.".to_string());
                }
                output.push_str(&format!("# {} Conventions\n\n", ws_name));
                for (name, desc) in &ws.conventions {
                    output.push_str(&format!("## {}\n{}\n\n", name, desc));
                }
            }
            Some("gotchas") => {
                if !has_gotchas {
                    return Ok("No workspace gotchas defined.".to_string());
                }
                output.push_str(&format!("# {} Gotchas\n\n", ws_name));
                for (name, desc) in &ws.gotchas {
                    output.push_str(&format!("## {}\n{}\n\n", name, desc));
                }
            }
            None => {
                if has_conventions {
                    output.push_str(&format!("# {} Conventions\n\n", ws_name));
                    for (name, desc) in &ws.conventions {
                        output.push_str(&format!("## {}\n{}\n\n", name, desc));
                    }
                }
                if has_gotchas {
                    output.push_str(&format!("# {} Gotchas\n\n", ws_name));
                    for (name, desc) in &ws.gotchas {
                        output.push_str(&format!("## {}\n{}\n\n", name, desc));
                    }
                }
            }
            Some(c) => return Err(format!("Unknown category '{}'. Use 'conventions' or 'gotchas'.", c)),
        }

        Ok(output)
    }
}

// ============================================================================
// Formatting Helpers
// ============================================================================

fn format_commands(commands: &HashMap<String, String>) -> String {
    if commands.is_empty() {
        return "No commands defined.".to_string();
    }
    let mut output = String::new();
    for (name, cmd) in commands {
        output.push_str(&format!("- **{}**: `{}`\n", name, cmd));
    }
    output
}

fn format_entry_points(entry_points: &HashMap<String, String>) -> String {
    if entry_points.is_empty() {
        return "No entry points defined.".to_string();
    }
    let mut output = String::new();
    for (name, path) in entry_points {
        output.push_str(&format!("- **{}**: {}\n", name, path));
    }
    output
}

fn format_dependencies(deps: &Dependencies) -> String {
    let mut output = String::new();
    if !deps.internal.is_empty() {
        output.push_str("**Internal dependencies:**\n");
        for dep in &deps.internal {
            output.push_str(&format!("- {}\n", dep));
        }
    }
    if !deps.external.is_empty() {
        output.push_str("**External dependencies:**\n");
        for dep in &deps.external {
            output.push_str(&format!("- {}\n", dep));
        }
    }
    if output.is_empty() {
        "No dependencies defined.".to_string()
    } else {
        output
    }
}

fn format_related_projects(related: &RelatedProjects) -> String {
    let mut output = String::new();
    if !related.upstream.is_empty() {
        output.push_str("**Upstream (this project depends on):**\n");
        for proj in &related.upstream {
            output.push_str(&format!("- {}\n", proj));
        }
    }
    if !related.downstream.is_empty() {
        output.push_str("**Downstream (depends on this project):**\n");
        for proj in &related.downstream {
            output.push_str(&format!("- {}\n", proj));
        }
    }
    if output.is_empty() {
        "No related projects defined.".to_string()
    } else {
        output
    }
}

fn format_api(api: &Option<ApiInfo>) -> String {
    match api {
        Some(api_info) => {
            let mut output = String::new();
            if let Some(openapi) = &api_info.openapi {
                output.push_str(&format!("**OpenAPI spec:** {}\n", openapi));
            }
            if let Some(base_url) = &api_info.base_url {
                output.push_str(&format!("**Base URL:** {}\n", base_url));
            }
            if !api_info.endpoints.is_empty() {
                output.push_str("**Endpoints:**\n");
                for endpoint in &api_info.endpoints {
                    output.push_str(&format!("- {}\n", endpoint));
                }
            }
            if output.is_empty() {
                "API section defined but empty.".to_string()
            } else {
                output
            }
        }
        None => "No API information defined.".to_string(),
    }
}

fn format_concept(project_path: &Path, name: &str, concept: &Concept) -> String {
    let mut output = format!("## {}\n\n{}\n\n**Files:**\n", name, concept.summary);
    for file in &concept.files {
        output.push_str(&format!("- {}/{}\n", project_path.display(), file));
    }
    output
}

// ============================================================================
// Main Loop
// ============================================================================

fn main() -> Result<()> {
    let args = Args::parse();

    let root = args
        .root
        .or_else(|| env::var("JUMBLE_ROOT").ok().map(PathBuf::from))
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let mut server = Server::new(root)?;

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = line.context("Failed to read from stdin")?;
        if line.is_empty() {
            continue;
        }

        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(e) => {
                let error_response = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: None,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700,
                        message: format!("Parse error: {}", e),
                        data: None,
                    }),
                };
                let response_json = serde_json::to_string(&error_response)?;
                writeln!(stdout, "{}", response_json)?;
                stdout.flush()?;
                continue;
            }
        };

        let response = server.handle_request(request);
        let response_json = serde_json::to_string(&response)?;
        writeln!(stdout, "{}", response_json)?;
        stdout.flush()?;
    }

    Ok(())
}
