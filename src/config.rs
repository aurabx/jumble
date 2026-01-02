//! Project and workspace configuration types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

// ============================================================================
// Project Configuration Types
// ============================================================================

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProjectConfig {
    pub project: ProjectInfo,
    #[serde(default)]
    pub commands: HashMap<String, String>,
    #[serde(default)]
    pub entry_points: HashMap<String, String>,
    #[serde(default)]
    pub dependencies: Dependencies,
    #[serde(default)]
    pub related_projects: RelatedProjects,
    #[serde(default)]
    pub api: Option<ApiInfo>,
    #[serde(default)]
    pub concepts: HashMap<String, Concept>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProjectInfo {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub repository: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Dependencies {
    #[serde(default)]
    pub internal: Vec<String>,
    #[serde(default)]
    pub external: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct RelatedProjects {
    #[serde(default)]
    pub upstream: Vec<String>,
    #[serde(default)]
    pub downstream: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApiInfo {
    #[serde(default)]
    pub openapi: Option<String>,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub endpoints: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Concept {
    pub files: Vec<String>,
    pub summary: String,
}

/// Optional YAML frontmatter for a skill file.
///
/// This mirrors the common `SKILL.md` / frontmatter pattern used by other tools:
///
/// ---
/// name: explaining-code
/// description: Explains code with visual diagrams and analogies
/// tags: [explain, diagram, analogy]
/// ---
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SkillFrontmatter {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Cached metadata for a single skill file.
#[derive(Debug, Clone)]
pub struct SkillInfo {
    /// Filesystem path to the skill markdown.
    pub path: PathBuf,
    /// Directory containing the skill and its companion files (scripts/, references/, etc.).
    /// For flat skills (e.g., .jumble/skills/my-skill.md), this is None.
    /// For SKILL.md files in directories, this is the parent directory.
    pub skill_dir: Option<PathBuf>,
    /// Optional parsed YAML frontmatter at the top of the file (between --- markers).
    pub frontmatter: Option<SkillFrontmatter>,
    /// A short preview snippet from the body of the skill (first few lines).
    pub preview: String,
}

/// Discovered skills for a project (from .jumble/skills/*.md)
#[derive(Debug, Clone, Default)]
pub struct ProjectSkills {
    /// Map from skill topic (file stem) to cached skill metadata.
    pub skills: HashMap<String, SkillInfo>,
}

/// Conventions and gotchas for a project (from .jumble/conventions.toml)
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ProjectConventions {
    #[serde(default)]
    pub conventions: HashMap<String, String>,
    #[serde(default)]
    pub gotchas: HashMap<String, String>,
}

/// Documentation index for a project (from .jumble/docs.toml)
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ProjectDocs {
    #[serde(default)]
    pub docs: HashMap<String, DocEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DocEntry {
    pub path: String,
    pub summary: String,
}

// ============================================================================
// Workspace Configuration (from .jumble/workspace.toml at root)
// ============================================================================

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct WorkspaceConfig {
    #[serde(default)]
    pub workspace: WorkspaceInfo,
    #[serde(default)]
    pub conventions: HashMap<String, String>,
    #[serde(default)]
    pub gotchas: HashMap<String, String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct WorkspaceInfo {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

// ============================================================================
// Global Jumble Configuration (~/.jumble/jumble.toml)
// ============================================================================

/// Global configuration loaded from `~/.jumble/jumble.toml` (or the
/// platform-specific equivalent of the user's home directory). This file is
/// created on startup if it does not exist and currently reserves a single
/// top-level `[jumble]` table for future options.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct JumbleConfig {
    #[serde(default)]
    pub jumble: JumbleSection,
}

/// Placeholder for future Jumble-wide options under the `[jumble]` table.
/// At the moment this is intentionally empty but ensures we always have a
/// well-typed location for future configuration.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct JumbleSection {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_project_config() {
        let toml_str = r#"
            [project]
            name = "test-project"
            description = "A test project"
        "#;

        let config: ProjectConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.project.name, "test-project");
        assert_eq!(config.project.description, "A test project");
        assert!(config.commands.is_empty());
        assert!(config.entry_points.is_empty());
        assert!(config.concepts.is_empty());
    }

    #[test]
    fn test_parse_full_project_config() {
        let toml_str = r#"
            [project]
            name = "my-app"
            description = "My application"
            language = "rust"
            version = "1.0.0"
            repository = "https://github.com/example/my-app"

            [commands]
            build = "cargo build"
            test = "cargo test"

            [entry_points]
            main = "src/main.rs"
            lib = "src/lib.rs"

            [dependencies]
            internal = ["shared-lib"]
            external = ["serde", "tokio"]

            [related_projects]
            upstream = ["core-lib"]
            downstream = ["web-frontend"]

            [api]
            openapi = "docs/openapi.yaml"
            base_url = "/api/v1"
            endpoints = ["GET /users", "POST /users"]

            [concepts.authentication]
            files = ["src/auth.rs", "src/jwt.rs"]
            summary = "JWT-based authentication"
        "#;

        let config: ProjectConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.project.name, "my-app");
        assert_eq!(config.project.language, Some("rust".to_string()));
        assert_eq!(config.commands.get("build"), Some(&"cargo build".to_string()));
        assert_eq!(config.entry_points.get("main"), Some(&"src/main.rs".to_string()));
        assert_eq!(config.dependencies.internal, vec!["shared-lib"]);
        assert_eq!(config.dependencies.external, vec!["serde", "tokio"]);
        assert_eq!(config.related_projects.upstream, vec!["core-lib"]);
        
        let api = config.api.unwrap();
        assert_eq!(api.openapi, Some("docs/openapi.yaml".to_string()));
        assert_eq!(api.endpoints.len(), 2);

        let auth_concept = config.concepts.get("authentication").unwrap();
        assert_eq!(auth_concept.files, vec!["src/auth.rs", "src/jwt.rs"]);
        assert_eq!(auth_concept.summary, "JWT-based authentication");
    }

    #[test]
    fn test_parse_workspace_config() {
        let toml_str = r#"
            [workspace]
            name = "my-workspace"
            description = "A monorepo workspace"

            [conventions]
            error_handling = "Use anyhow for application errors"
            logging = "Use tracing for structured logging"

            [gotchas]
            async_deadlock = "Avoid holding locks across await points"
        "#;

        let config: WorkspaceConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.workspace.name, Some("my-workspace".to_string()));
        assert_eq!(config.conventions.len(), 2);
        assert_eq!(config.gotchas.len(), 1);
        assert!(config.gotchas.contains_key("async_deadlock"));
    }

    #[test]
    fn test_parse_conventions() {
        let toml_str = r#"
            [conventions]
            naming = "Use snake_case for functions"

            [gotchas]
            null_check = "Always check for None"
        "#;

        let conventions: ProjectConventions = toml::from_str(toml_str).unwrap();
        assert_eq!(conventions.conventions.get("naming"), Some(&"Use snake_case for functions".to_string()));
        assert_eq!(conventions.gotchas.get("null_check"), Some(&"Always check for None".to_string()));
    }

    #[test]
    fn test_parse_docs() {
        let toml_str = r#"
            [docs.architecture]
            path = "docs/architecture.md"
            summary = "System architecture overview"

            [docs.api]
            path = "docs/api.md"
            summary = "API reference documentation"
        "#;

        let docs: ProjectDocs = toml::from_str(toml_str).unwrap();
        assert_eq!(docs.docs.len(), 2);
        
        let arch_doc = docs.docs.get("architecture").unwrap();
        assert_eq!(arch_doc.path, "docs/architecture.md");
        assert_eq!(arch_doc.summary, "System architecture overview");
    }

    #[test]
    fn test_defaults_for_missing_fields() {
        let toml_str = r#"
            [project]
            name = "minimal"
            description = "Minimal config"
        "#;

        let config: ProjectConfig = toml::from_str(toml_str).unwrap();
        assert!(config.project.language.is_none());
        assert!(config.project.version.is_none());
        assert!(config.api.is_none());
        assert!(config.dependencies.internal.is_empty());
        assert!(config.dependencies.external.is_empty());
    }
}
