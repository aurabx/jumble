//! MCP Server implementation.

use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::config::{
    JumbleConfig,
    ProjectConfig,
    ProjectConventions,
    ProjectDocs,
    ProjectSkills,
    SkillFrontmatter,
    SkillInfo,
    WorkspaceConfig,
};
use crate::protocol::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use crate::tools::{self, ProjectData};

/// MCP Server state
pub struct Server {
    pub root: PathBuf,
    pub workspace: Option<WorkspaceConfig>,
    pub projects: HashMap<String, ProjectData>,
    /// Global Jumble configuration loaded from `~/.jumble/jumble.toml`.
    #[allow(dead_code)]
    pub jumble_config: Option<JumbleConfig>,
}

impl Server {
    pub fn new(root: PathBuf) -> Result<Self> {
        let mut server = Server {
            root,
            workspace: None,
            projects: HashMap::new(),
            jumble_config: load_jumble_config(),
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

                    // Discover skills, conventions, and docs
                    let skills = self.discover_skills(path.parent().unwrap());
                    let conventions = self.load_conventions(path.parent().unwrap());
                    let docs = self.load_docs(path.parent().unwrap());

                    projects.insert(
                        config.project.name.clone(),
                        (project_dir, config, skills, conventions, docs),
                    );
                }
            }
        }
        Ok(projects)
    }

    fn discover_skills(&self, jumble_dir: &Path) -> ProjectSkills {
        let mut skills = ProjectSkills::default();
        let skills_dir = jumble_dir.join("skills");

        // Traditional project-local .jumble/skills/*.md files
        if skills_dir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&skills_dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.extension().map(|e| e == "md").unwrap_or(false) {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            let (frontmatter, preview) = match std::fs::read_to_string(&path) {
                                Ok(content) => extract_skill_frontmatter_and_preview(&content),
                                Err(_) => (None, String::new()),
                            };

                            skills.skills.insert(
                                stem.to_string(),
                                SkillInfo {
                                    path: path.clone(),
                                    skill_dir: None, // Flat skills have no companion directory
                                    frontmatter,
                                    preview,
                                },
                            );
                        }
                    }
                }
            }
        }

        // Personal/global Jumble skills: <home>/.jumble/skills/*.md
        if let Some(home_dir) = resolve_home_dir() {
            let global_skills_dir = home_dir.join(".jumble").join("skills");
            if global_skills_dir.is_dir() {
                if let Ok(entries) = std::fs::read_dir(&global_skills_dir) {
                    for entry in entries.filter_map(|e| e.ok()) {
                        let path = entry.path();
                        if path.extension().map(|e| e == "md").unwrap_or(false) {
                            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                                // Don't override project-local skills with global ones.
                                if skills.skills.contains_key(stem) {
                                    continue;
                                }

                                let (frontmatter, preview) = match std::fs::read_to_string(&path) {
                                    Ok(content) => extract_skill_frontmatter_and_preview(&content),
                                    Err(_) => (None, String::new()),
                                };

                                skills.skills.insert(
                                    stem.to_string(),
                                    SkillInfo {
                                        path: path.clone(),
                                        skill_dir: None,
                                        frontmatter,
                                        preview,
                                    },
                                );
                            }
                        }
                    }
                }
            }
        }

        // Project-local Claude skills: <project_root>/.claude/skills/**/SKILL.md
        if let Some(project_root) = jumble_dir.parent() {
            let claude_skills_dir = project_root.join(".claude/skills");
            if claude_skills_dir.is_dir() {
                discover_structured_skills_in_dir(&claude_skills_dir, &mut skills);
            }
        }

        // Personal/global Claude skills: <home>/.claude/skills/**/SKILL.md
        if let Some(home_dir) = resolve_home_dir() {
            let personal_skills_dir = home_dir.join(".claude/skills");
            if personal_skills_dir.is_dir() {
                discover_structured_skills_in_dir(&personal_skills_dir, &mut skills);
            }
        }

        // Project-local Codex skills: <project_root>/.codex/skills/**/SKILL.md
        if let Some(project_root) = jumble_dir.parent() {
            let codex_skills_dir = project_root.join(".codex/skills");
            if codex_skills_dir.is_dir() {
                discover_structured_skills_in_dir(&codex_skills_dir, &mut skills);
            }
        }

        // Personal/global Codex skills: <home>/.codex/skills/**/SKILL.md
        if let Some(home_dir) = resolve_home_dir() {
            let personal_codex_dir = home_dir.join(".codex/skills");
            if personal_codex_dir.is_dir() {
                discover_structured_skills_in_dir(&personal_codex_dir, &mut skills);
            }
        }

        skills
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
            "list_skills" => tools::list_skills(&self.projects, &arguments),
            "get_skill" => tools::get_skill(&self.projects, &arguments),
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

/// Resolve the current user's home directory in a cross-platform way.
///
/// On Unix-like systems this prefers the `HOME` environment variable. On
/// Windows it falls back to `USERPROFILE`, then `HOMEDRIVE` + `HOMEPATH`.
fn resolve_home_dir() -> Option<PathBuf> {
    if let Ok(home) = std::env::var("HOME") {
        if !home.is_empty() {
            return Some(PathBuf::from(home));
        }
    }

    if let Ok(profile) = std::env::var("USERPROFILE") {
        if !profile.is_empty() {
            return Some(PathBuf::from(profile));
        }
    }

    if let (Ok(drive), Ok(path)) = (std::env::var("HOMEDRIVE"), std::env::var("HOMEPATH")) {
        let combined = format!("{}{}", drive, path);
        if !combined.is_empty() {
            return Some(PathBuf::from(combined));
        }
    }

    None
}

/// Load global Jumble configuration from `~/.jumble/jumble.toml`, creating a
/// default file if it does not exist. Failures to read or parse the file are
/// logged to stderr but do not prevent the server from starting.
fn load_jumble_config() -> Option<JumbleConfig> {
    let home_dir = resolve_home_dir()?;
    let jumble_dir = home_dir.join(".jumble");
    let config_path = jumble_dir.join("jumble.toml");

    if !config_path.exists() {
        if let Err(e) = std::fs::create_dir_all(&jumble_dir) {
            eprintln!(
                "jumble: failed to create global config directory at {}: {}",
                jumble_dir.display(),
                e
            );
            return None;
        }

        let default_content = "# Global configuration for the Jumble MCP server.\n\n[jumble]\n";
        if let Err(e) = std::fs::write(&config_path, default_content) {
            eprintln!(
                "jumble: failed to create default config at {}: {}",
                config_path.display(),
                e
            );
            return None;
        }
    }

    let content = match std::fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!(
                "jumble: failed to read global config at {}: {}",
                config_path.display(),
                e
            );
            return None;
        }
    };

    match toml::from_str::<JumbleConfig>(&content) {
        Ok(cfg) => Some(cfg),
        Err(e) => {
            eprintln!(
                "jumble: failed to parse global config at {}: {}",
                config_path.display(),
                e
            );
            None
        }
    }
}

/// Discover structured skills (Claude/Codex-style) with SKILL.md files and companion resources.
fn discover_structured_skills_in_dir(root: &Path, skills: &mut ProjectSkills) {
    for entry in WalkDir::new(root)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        // Structured skills (Claude/Codex) conventionally use `SKILL.md` as the filename.
        let is_skill = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.eq_ignore_ascii_case("SKILL.md"))
            .unwrap_or(false);
        if !is_skill {
            continue;
        }

        let (frontmatter, preview) = match std::fs::read_to_string(path) {
            Ok(content) => extract_skill_frontmatter_and_preview(&content),
            Err(_) => (None, String::new()),
        };

        // Determine the skill key. Prefer the frontmatter `name` field when present,
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

        if key.is_empty() || skills.skills.contains_key(&key) {
            // Skip empty keys and avoid overwriting existing skills from .jumble/skills.
            continue;
        }

        // Store the skill directory (parent of SKILL.md) for companion file access
        let skill_directory = path.parent().map(|p| p.to_path_buf());

        skills.skills.insert(
            key,
            SkillInfo {
                path: path.to_path_buf(),
                skill_dir: skill_directory,
                frontmatter,
                preview,
            },
        );
    }
}

/// Extract optional YAML frontmatter and a preview snippet from a skill file.
///
/// Frontmatter is only recognized when the file starts with a line containing only `---`.
/// Everything between the first and second such markers is treated as YAML.
/// The preview is taken from the body that follows the frontmatter (or from the
/// top of the file when no frontmatter is present).
fn extract_skill_frontmatter_and_preview(
    content: &str,
) -> (Option<SkillFrontmatter>, String) {
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

            let frontmatter = serde_yaml::from_str::<SkillFrontmatter>(frontmatter_str).ok();
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
    use std::collections::HashMap;

    #[test]
    fn test_extract_frontmatter_and_preview_with_valid_frontmatter() {
        let content = "---\nname: bootstrap\ndescription: Test description\ntags: [a, b]\n---\n# Title\nBody line 1\nBody line 2\n";

        let (frontmatter, preview) = extract_skill_frontmatter_and_preview(content);

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

        let (frontmatter, preview) = extract_skill_frontmatter_and_preview(content);

        assert!(frontmatter.is_none());
        // Preview should include the top of the file when no frontmatter exists.
        assert!(preview.starts_with("# Title"));
        assert!(preview.contains("Line 1"));
    }

    #[test]
    fn test_extract_frontmatter_and_preview_with_unclosed_frontmatter() {
        // Starts with `---` but has no closing marker; this should fall back to no frontmatter.
        let content = "---\nname: broken\n# Title\nLine 1\n";

        let (frontmatter, preview) = extract_skill_frontmatter_and_preview(content);

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

        let mut skills = ProjectSkills::default();
        discover_structured_skills_in_dir(&tmp_root, &mut skills);

        // Clean up best-effort; ignore failures.
        let _ = std::fs::remove_dir_all(&tmp_root);

        let info = skills
            .skills
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

        let mut skills = ProjectSkills::default();
        discover_structured_skills_in_dir(&tmp_root, &mut skills);

        let _ = std::fs::remove_dir_all(&tmp_root);

        let info = skills
            .skills
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

    #[test]
    fn test_resolve_home_dir_and_global_jumble_skills() {
        use std::env;

        // Save original environment so we can restore after the test.
        let orig_home = env::var("HOME").ok();
        let orig_userprofile = env::var("USERPROFILE").ok();
        let orig_homedrive = env::var("HOMEDRIVE").ok();
        let orig_homepath = env::var("HOMEPATH").ok();

        // Use a temporary directory as our synthetic home.
        let tmp_root = std::env::temp_dir().join("jumble_test_home_global_skills");
        let _ = std::fs::remove_dir_all(&tmp_root);
        std::fs::create_dir_all(&tmp_root).unwrap();

        env::set_var("HOME", &tmp_root);
        env::remove_var("USERPROFILE");
        env::remove_var("HOMEDRIVE");
        env::remove_var("HOMEPATH");

        let home = resolve_home_dir().expect("expected home directory");
        assert_eq!(home, tmp_root);

        // Loading global Jumble config should create ~/.jumble/jumble.toml if missing.
        let cfg = load_jumble_config();
        let cfg_path = home.join(".jumble").join("jumble.toml");
        assert!(cfg_path.exists());
        assert!(cfg.is_some());

        // Global Jumble skills live in <home>/.jumble/skills/*.md
        let global_skills_dir = home.join(".jumble").join("skills");
        std::fs::create_dir_all(&global_skills_dir).unwrap();
        let global_skill_path = global_skills_dir.join("global-skill.md");
        std::fs::write(&global_skill_path, "# Global Skill\\nBody").unwrap();

        // Create a fake project with a .jumble directory.
        let project_root = home.join("workspace").join("my-project");
        let jumble_dir = project_root.join(".jumble");
        let project_skills_dir = jumble_dir.join("skills");
        std::fs::create_dir_all(&project_skills_dir).unwrap();

        // Local skill with the same stem as a global one should win.
        let local_first_path = project_skills_dir.join("local-first.md");
        std::fs::write(&local_first_path, "# Local First\\nBody").unwrap();
        let global_conflict_path = global_skills_dir.join("local-first.md");
        std::fs::write(&global_conflict_path, "# Global Conflict\\nBody").unwrap();

        let server = Server {
            root: project_root.clone(),
            workspace: None,
            projects: HashMap::new(),
            jumble_config: cfg,
        };

        let skills = server.discover_skills(&jumble_dir);

        // Global-only skill should be present and loaded from the global path.
        let global_info = skills
            .skills
            .get("global-skill")
            .expect("expected global skill discovered");
        assert_eq!(global_info.path, global_skill_path);

        // For a conflicting name, the project-local skill should take precedence.
        let local_info = skills
            .skills
            .get("local-first")
            .expect("expected local-first skill discovered");
        assert_eq!(local_info.path, local_first_path);

        // Best-effort cleanup; ignore failures.
        let _ = std::fs::remove_dir_all(&tmp_root);

        // Restore original environment.
        match orig_home {
            Some(v) => env::set_var("HOME", v),
            None => env::remove_var("HOME"),
        }
        match orig_userprofile {
            Some(v) => env::set_var("USERPROFILE", v),
            None => env::remove_var("USERPROFILE"),
        }
        match orig_homedrive {
            Some(v) => env::set_var("HOMEDRIVE", v),
            None => env::remove_var("HOMEDRIVE"),
        }
        match orig_homepath {
            Some(v) => env::set_var("HOMEPATH", v),
            None => env::remove_var("HOMEPATH"),
        }
    }
}
