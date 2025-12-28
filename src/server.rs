//! MCP Server implementation.

use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::config::{
    ProjectConfig, ProjectConventions, ProjectDocs, ProjectPrompts, WorkspaceConfig,
};
use crate::protocol::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use crate::tools::{self, ProjectData};

/// MCP Server state
pub struct Server {
    pub root: PathBuf,
    pub workspace: Option<WorkspaceConfig>,
}

impl Server {
    pub fn new(root: PathBuf) -> Result<Self> {
        let workspace = Self::load_workspace_static(&root);
        let server = Server {
            root,
            workspace,
        };
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

    fn discover_projects(&self) -> Result<HashMap<String, ProjectData>> {
        let mut projects = HashMap::new();
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

                    projects.insert(
                        config.project.name.clone(),
                        (project_dir, config, prompts, conventions, docs),
                    );
                }
            }
        }
        Ok(projects)
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

    pub fn handle_request(&mut self, request: JsonRpcRequest) -> JsonRpcResponse {
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
            Ok(value) => JsonRpcResponse::success(request.id, value),
            Err(error) => JsonRpcResponse::error(request.id, error),
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
        Ok(tools::tools_list())
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

        // Rebuild projects on each call for hot reloading
        let projects = self.discover_projects().map_err(|e| JsonRpcError {
            code: -32603,
            message: format!("Failed to discover projects: {}", e),
            data: None,
        })?;

        let result = match name {
            "list_projects" => tools::list_projects(&projects),
            "get_project_info" => tools::get_project_info(&projects, &arguments),
            "get_commands" => tools::get_commands(&projects, &arguments),
            "get_architecture" => tools::get_architecture(&projects, &arguments),
            "get_related_files" => tools::get_related_files(&projects, &arguments),
            "list_prompts" => tools::list_prompts(&projects, &arguments),
            "get_prompt" => tools::get_prompt(&projects, &arguments),
            "get_conventions" => tools::get_conventions(&projects, &arguments),
            "get_docs" => tools::get_docs(&projects, &arguments),
            "get_workspace_overview" => {
                tools::get_workspace_overview(&self.root, &self.workspace, &projects)
            }
            "get_workspace_conventions" => {
                tools::get_workspace_conventions(&self.workspace, &arguments)
            }
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
}
