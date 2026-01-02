//! MCP Server implementation.

use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::config::{
    ProjectConfig,
    ProjectConventions,
    ProjectDocs,
    ProjectPrompts,
    PromptFrontmatter,
    PromptInfo,
    WorkspaceConfig,
};
use crate::protocol::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use crate::tools::{self, ProjectData};

/// MCP Server state
pub struct Server {
    pub root: PathBuf,
    pub workspace: Option<WorkspaceConfig>,
    pub projects: HashMap<String, ProjectData>,
}

impl Server {
    pub fn new(root: PathBuf) -> Result<Self> {
        let mut server = Server {
            root,
            workspace: None,
            projects: HashMap::new(),
        };
        server.reload_workspace_and_projects()?;
        Ok(server)
    }

    fn reload_workspace_and_projects(&mut self) -> Result<()> {
        self.workspace = Self::load_workspace_static(&self.root);
        self.projects = self.discover_projects()?;
        Ok(())
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

        // Traditional .jumble/prompts/*.md files
        if prompts_dir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&prompts_dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.extension().map(|e| e == "md").unwrap_or(false) {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            let (frontmatter, preview) = match std::fs::read_to_string(&path) {
                                Ok(content) => extract_prompt_frontmatter_and_preview(&content),
                                Err(_) => (None, String::new()),
                            };

                            prompts.prompts.insert(
                                stem.to_string(),
                                PromptInfo {
                                    path: path.clone(),
                                    frontmatter,
                                    preview,
                                },
                            );
                        }
                    }
                }
            }
        }

        // Project-local Claude skills: <project_root>/.claude/skills/**/SKILL.md
        if let Some(project_root) = jumble_dir.parent() {
            let claude_skills_dir = project_root.join(".claude/skills");
            if claude_skills_dir.is_dir() {
                discover_claude_skills_in_dir(&claude_skills_dir, &mut prompts);
            }
        }

        // Personal/global Claude skills: $HOME/.claude/skills/**/SKILL.md
        if let Ok(home) = std::env::var("HOME") {
            let personal_skills_dir = Path::new(&home).join(".claude/skills");
            if personal_skills_dir.is_dir() {
                discover_claude_skills_in_dir(&personal_skills_dir, &mut prompts);
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

        let result = match name {
            "reload_workspace" => match self.reload_workspace_and_projects() {
                Ok(()) => Ok("Workspace and projects reloaded from disk.".to_string()),
                Err(e) => Err(format!("Failed to reload workspace: {}", e)),
            },
            "list_projects" => tools::list_projects(&self.projects),
            "get_project_info" => tools::get_project_info(&self.projects, &arguments),
            "get_commands" => tools::get_commands(&self.projects, &arguments),
            "get_architecture" => tools::get_architecture(&self.projects, &arguments),
            "get_related_files" => tools::get_related_files(&self.projects, &arguments),
            "list_prompts" => tools::list_prompts(&self.projects, &arguments),
            "get_prompt" => tools::get_prompt(&self.projects, &arguments),
            "get_conventions" => tools::get_conventions(&self.projects, &arguments),
            "get_docs" => tools::get_docs(&self.projects, &arguments),
            "get_workspace_overview" => {
                tools::get_workspace_overview(&self.root, &self.workspace, &self.projects)
            }
            "get_workspace_conventions" => {
                tools::get_workspace_conventions(&self.workspace, &arguments)
            }
            "get_jumble_authoring_prompt" => tools::get_jumble_authoring_prompt(),
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

/// Discover Claude-style SKILL.md files under a skills root and add them as prompts.
fn discover_claude_skills_in_dir(root: &Path, prompts: &mut ProjectPrompts) {
    for entry in WalkDir::new(root)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        // Claude skills conventionally use `SKILL.md` as the filename.
        let is_skill = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.eq_ignore_ascii_case("SKILL.md"))
            .unwrap_or(false);
        if !is_skill {
            continue;
        }

        let (frontmatter, preview) = match std::fs::read_to_string(path) {
            Ok(content) => extract_prompt_frontmatter_and_preview(&content),
            Err(_) => (None, String::new()),
        };

        // Determine the prompt key. Prefer the frontmatter `name` field when present,
        // otherwise fall back to the containing directory name.
        let mut key = frontmatter
            .as_ref()
            .and_then(|fm| fm.name.clone())
            .unwrap_or_default();

        if key.is_empty() {
            key = path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("skill")
                .to_string();
        }

        if key.is_empty() || prompts.prompts.contains_key(&key) {
            // Skip empty keys and avoid overwriting existing prompts from .jumble/prompts.
            continue;
        }

        prompts.prompts.insert(
            key,
            PromptInfo {
                path: path.to_path_buf(),
                frontmatter,
                preview,
            },
        );
    }
}

/// Extract optional YAML frontmatter and a preview snippet from a prompt file.
///
/// Frontmatter is only recognized when the file starts with a line containing only `---`.
/// Everything between the first and second such markers is treated as YAML.
/// The preview is taken from the body that follows the frontmatter (or from the
/// top of the file when no frontmatter is present).
fn extract_prompt_frontmatter_and_preview(
    content: &str,
) -> (Option<PromptFrontmatter>, String) {
    const PREVIEW_MAX_LINES: usize = 16;

    // Helper to build a preview from a body slice.
    fn build_preview(body: &str) -> String {
        body
            .lines()
            .take(PREVIEW_MAX_LINES)
            .collect::<Vec<_>>()
            .join("\n")
    }

    // Detect YAML frontmatter only if the file starts with `---` on the first line.
    if content.starts_with("---\n") {
        // Skip the initial `---\n`.
        let rest = &content[4..];
        if let Some(end_idx) = rest.find("\n---\n") {
            let frontmatter_str = &rest[..end_idx];
            let body_start = end_idx + "\n---\n".len();
            let body = &rest[body_start..];

            let frontmatter = serde_yaml::from_str::<PromptFrontmatter>(frontmatter_str).ok();
            let preview = build_preview(body);
            return (frontmatter, preview);
        }
    }

    // No valid frontmatter header found; fall back to using the first lines of the file.
    (None, build_preview(content))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_frontmatter_and_preview_with_valid_frontmatter() {
        let content = "---\nname: bootstrap\ndescription: Test description\ntags: [a, b]\n---\n# Title\nBody line 1\nBody line 2\n";

        let (frontmatter, preview) = extract_prompt_frontmatter_and_preview(content);

        let fm = frontmatter.expect("expected some frontmatter");
        assert_eq!(fm.name.as_deref(), Some("bootstrap"));
        assert_eq!(fm.description.as_deref(), Some("Test description"));
        assert_eq!(fm.tags, vec!["a", "b"]);

        // Preview should be built from the body after the closing `---`.
        assert!(preview.starts_with("# Title"));
        assert!(preview.contains("Body line 1"));
    }

    #[test]
    fn test_extract_frontmatter_and_preview_without_frontmatter() {
        let content = "# Title\nLine 1\nLine 2\n";

        let (frontmatter, preview) = extract_prompt_frontmatter_and_preview(content);

        assert!(frontmatter.is_none());
        // Preview should include the top of the file when no frontmatter exists.
        assert!(preview.starts_with("# Title"));
        assert!(preview.contains("Line 1"));
    }

    #[test]
    fn test_extract_frontmatter_and_preview_with_unclosed_frontmatter() {
        // Starts with `---` but has no closing marker; this should fall back to no frontmatter.
        let content = "---\nname: broken\n# Title\nLine 1\n";

        let (frontmatter, preview) = extract_prompt_frontmatter_and_preview(content);

        assert!(frontmatter.is_none());
        // In this failure mode we currently treat the whole file as body for the preview.
        assert!(preview.starts_with("---"));
        assert!(preview.contains("name: broken"));
    }

    #[test]
    fn test_discover_claude_skills_uses_frontmatter_name() {
        // Create a temporary skills directory structure:
        // <tmp>/skills/explaining-code/SKILL.md
        let tmp_root = std::env::temp_dir().join("jumble_test_skills_frontmatter");
        let skill_dir = tmp_root.join("explaining-code");
        std::fs::create_dir_all(&skill_dir).unwrap();

        let skill_path = skill_dir.join("SKILL.md");
        let content = "---\nname: explaining-code\ndescription: Explains code with diagrams\n---\nBody";
        std::fs::write(&skill_path, content).unwrap();

        let mut prompts = ProjectPrompts::default();
        discover_claude_skills_in_dir(&tmp_root, &mut prompts);

        // Clean up best-effort; ignore failures.
        let _ = std::fs::remove_dir_all(&tmp_root);

        let info = prompts
            .prompts
            .get("explaining-code")
            .expect("expected skill discovered with name from frontmatter");
        assert_eq!(info.path, skill_path);
        let fm = info
            .frontmatter
            .as_ref()
            .expect("expected parsed frontmatter");
        assert_eq!(fm.name.as_deref(), Some("explaining-code"));
        assert_eq!(fm.description.as_deref(), Some("Explains code with diagrams"));
    }

    #[test]
    fn test_discover_claude_skills_falls_back_to_dir_name_when_no_name() {
        // Create a temporary skills directory structure:
        // <tmp>/skills/diagramming/SKILL.md (no `name` field in frontmatter)
        let tmp_root = std::env::temp_dir().join("jumble_test_skills_dirname");
        let skill_dir = tmp_root.join("diagramming");
        std::fs::create_dir_all(&skill_dir).unwrap();

        let skill_path = skill_dir.join("SKILL.md");
        let content = "---\ndescription: Diagramming helper\n---\nBody";
        std::fs::write(&skill_path, content).unwrap();

        let mut prompts = ProjectPrompts::default();
        discover_claude_skills_in_dir(&tmp_root, &mut prompts);

        let _ = std::fs::remove_dir_all(&tmp_root);

        let info = prompts
            .prompts
            .get("diagramming")
            .expect("expected skill discovered with key from directory name");
        assert_eq!(info.path, skill_path);
        let fm = info
            .frontmatter
            .as_ref()
            .expect("expected parsed frontmatter even without name");
        assert_eq!(fm.name, None);
        assert_eq!(fm.description.as_deref(), Some("Diagramming helper"));
    }
}
