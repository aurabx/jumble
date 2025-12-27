//! MCP tool implementations.

use crate::config::{
    Concept, ProjectConfig, ProjectConventions, ProjectDocs, ProjectPrompts, WorkspaceConfig,
};
use crate::format::{
    format_api, format_commands, format_concept, format_dependencies, format_entry_points,
    format_related_projects,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;

/// Type alias for project data stored in the server
pub type ProjectData = (
    PathBuf,
    ProjectConfig,
    ProjectPrompts,
    ProjectConventions,
    ProjectDocs,
);

/// Returns the JSON schema for all available tools
pub fn tools_list() -> Value {
    json!({
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
    })
}

// ============================================================================
// Tool Implementations
// ============================================================================

pub fn list_projects(projects: &HashMap<String, ProjectData>) -> Result<String, String> {
    if projects.is_empty() {
        return Ok(
            "No projects found. Make sure .jumble/project.toml files exist in your workspace."
                .to_string(),
        );
    }

    let mut output = String::new();
    for (name, (path, config, _prompts, _conventions, _docs)) in projects {
        let lang = config.project.language.as_deref().unwrap_or("unknown");
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

pub fn get_project_info(
    projects: &HashMap<String, ProjectData>,
    args: &Value,
) -> Result<String, String> {
    let project_name = args
        .get("project")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'project' argument")?;

    let (path, config, _prompts, _conventions, _docs) = projects
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

pub fn get_commands(
    projects: &HashMap<String, ProjectData>,
    args: &Value,
) -> Result<String, String> {
    let project_name = args
        .get("project")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'project' argument")?;

    let (_, config, _, _, _) = projects
        .get(project_name)
        .ok_or_else(|| format!("Project '{}' not found", project_name))?;

    let command_type = args.get("command_type").and_then(|v| v.as_str());

    match command_type {
        Some(cmd_type) => config
            .commands
            .get(cmd_type)
            .map(|cmd| format!("{}: {}", cmd_type, cmd))
            .ok_or_else(|| {
                format!(
                    "Command '{}' not found for project '{}'",
                    cmd_type, project_name
                )
            }),
        None => Ok(format_commands(&config.commands)),
    }
}

pub fn get_architecture(
    projects: &HashMap<String, ProjectData>,
    args: &Value,
) -> Result<String, String> {
    let project_name = args
        .get("project")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'project' argument")?;

    let concept_name = args
        .get("concept")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'concept' argument")?;

    let (path, config, _, _, _) = projects
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

pub fn get_related_files(
    projects: &HashMap<String, ProjectData>,
    args: &Value,
) -> Result<String, String> {
    let project_name = args
        .get("project")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'project' argument")?;

    let query = args
        .get("query")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'query' argument")?;

    let (path, config, _, _, _) = projects
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

pub fn list_prompts(
    projects: &HashMap<String, ProjectData>,
    args: &Value,
) -> Result<String, String> {
    let project_name = args
        .get("project")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'project' argument")?;

    let (_, _, prompts, _, _) = projects
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

pub fn get_prompt(
    projects: &HashMap<String, ProjectData>,
    args: &Value,
) -> Result<String, String> {
    let project_name = args
        .get("project")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'project' argument")?;

    let topic = args
        .get("topic")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'topic' argument")?;

    let (_, _, prompts, _, _) = projects
        .get(project_name)
        .ok_or_else(|| format!("Project '{}' not found", project_name))?;

    let prompt_path = prompts.prompts.get(topic).ok_or_else(|| {
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

    std::fs::read_to_string(prompt_path).map_err(|e| format!("Failed to read prompt: {}", e))
}

pub fn get_conventions(
    projects: &HashMap<String, ProjectData>,
    args: &Value,
) -> Result<String, String> {
    let project_name = args
        .get("project")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'project' argument")?;

    let category = args.get("category").and_then(|v| v.as_str());

    let (_, _, _, conventions, _) = projects
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
        Some(c) => {
            return Err(format!(
                "Unknown category '{}'. Use 'conventions' or 'gotchas'.",
                c
            ))
        }
    }

    Ok(output)
}

pub fn get_docs(projects: &HashMap<String, ProjectData>, args: &Value) -> Result<String, String> {
    let project_name = args
        .get("project")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'project' argument")?;

    let topic = args.get("topic").and_then(|v| v.as_str());

    let (path, _, _, _, docs) = projects
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
                t,
                doc.summary,
                full_path.display()
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

pub fn get_workspace_overview(
    root: &std::path::Path,
    workspace: &Option<WorkspaceConfig>,
    projects: &HashMap<String, ProjectData>,
) -> Result<String, String> {
    let mut output = String::new();

    // Workspace info
    if let Some(ws) = workspace {
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

    output.push_str(&format!("**Root:** {}\n\n", root.display()));

    // Projects list
    if projects.is_empty() {
        output.push_str("No projects found.\n");
        return Ok(output);
    }

    output.push_str("## Projects\n\n");

    // Collect and sort projects for consistent output
    let mut project_names: Vec<&String> = projects.keys().collect();
    project_names.sort();

    for name in &project_names {
        let (_, config, _, _, _) = projects.get(*name).unwrap();
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
        let (_, config, _, _, _) = projects.get(*name).unwrap();
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
    if workspace.is_some() {
        output.push_str("\n*Use get_workspace_conventions() for workspace-wide coding standards.*");
    }

    Ok(output)
}

pub fn get_workspace_conventions(
    workspace: &Option<WorkspaceConfig>,
    args: &Value,
) -> Result<String, String> {
    let ws = workspace.as_ref().ok_or(
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
        Some(c) => {
            return Err(format!(
                "Unknown category '{}'. Use 'conventions' or 'gotchas'.",
                c
            ))
        }
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::*;
    use std::path::PathBuf;

    fn create_test_project() -> (String, ProjectData) {
        let config = ProjectConfig {
            project: ProjectInfo {
                name: "test-project".to_string(),
                description: "A test project".to_string(),
                language: Some("rust".to_string()),
                version: Some("1.0.0".to_string()),
                repository: None,
            },
            commands: {
                let mut map = HashMap::new();
                map.insert("build".to_string(), "cargo build".to_string());
                map.insert("test".to_string(), "cargo test".to_string());
                map
            },
            entry_points: {
                let mut map = HashMap::new();
                map.insert("main".to_string(), "src/main.rs".to_string());
                map
            },
            dependencies: Dependencies {
                internal: vec!["shared".to_string()],
                external: vec!["serde".to_string()],
            },
            related_projects: RelatedProjects {
                upstream: vec!["core".to_string()],
                downstream: vec![],
            },
            api: None,
            concepts: {
                let mut map = HashMap::new();
                map.insert(
                    "authentication".to_string(),
                    Concept {
                        files: vec!["src/auth.rs".to_string()],
                        summary: "JWT auth".to_string(),
                    },
                );
                map
            },
        };

        let prompts = ProjectPrompts::default();
        let conventions = ProjectConventions {
            conventions: {
                let mut map = HashMap::new();
                map.insert("naming".to_string(), "Use snake_case".to_string());
                map
            },
            gotchas: {
                let mut map = HashMap::new();
                map.insert("async".to_string(), "Avoid blocking".to_string());
                map
            },
        };
        let docs = ProjectDocs {
            docs: {
                let mut map = HashMap::new();
                map.insert(
                    "readme".to_string(),
                    DocEntry {
                        path: "README.md".to_string(),
                        summary: "Project readme".to_string(),
                    },
                );
                map
            },
        };

        (
            "test-project".to_string(),
            (PathBuf::from("/test"), config, prompts, conventions, docs),
        )
    }

    fn create_test_projects() -> HashMap<String, ProjectData> {
        let mut projects = HashMap::new();
        let (name, data) = create_test_project();
        projects.insert(name, data);
        projects
    }

    #[test]
    fn test_list_projects_empty() {
        let projects = HashMap::new();
        let result = list_projects(&projects).unwrap();
        assert!(result.contains("No projects found"));
    }

    #[test]
    fn test_list_projects() {
        let projects = create_test_projects();
        let result = list_projects(&projects).unwrap();
        assert!(result.contains("test-project"));
        assert!(result.contains("rust"));
        assert!(result.contains("A test project"));
    }

    #[test]
    fn test_get_project_info_not_found() {
        let projects = create_test_projects();
        let args = json!({"project": "nonexistent"});
        let result = get_project_info(&projects, &args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_get_project_info_full() {
        let projects = create_test_projects();
        let args = json!({"project": "test-project"});
        let result = get_project_info(&projects, &args).unwrap();
        assert!(result.contains("test-project"));
        assert!(result.contains("A test project"));
        assert!(result.contains("rust"));
        assert!(result.contains("1.0.0"));
    }

    #[test]
    fn test_get_project_info_commands_field() {
        let projects = create_test_projects();
        let args = json!({"project": "test-project", "field": "commands"});
        let result = get_project_info(&projects, &args).unwrap();
        assert!(result.contains("build"));
        assert!(result.contains("cargo build"));
    }

    #[test]
    fn test_get_commands() {
        let projects = create_test_projects();
        let args = json!({"project": "test-project"});
        let result = get_commands(&projects, &args).unwrap();
        assert!(result.contains("build"));
        assert!(result.contains("test"));
    }

    #[test]
    fn test_get_commands_specific() {
        let projects = create_test_projects();
        let args = json!({"project": "test-project", "command_type": "build"});
        let result = get_commands(&projects, &args).unwrap();
        assert!(result.contains("cargo build"));
    }

    #[test]
    fn test_get_commands_not_found() {
        let projects = create_test_projects();
        let args = json!({"project": "test-project", "command_type": "deploy"});
        let result = get_commands(&projects, &args);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_architecture() {
        let projects = create_test_projects();
        let args = json!({"project": "test-project", "concept": "authentication"});
        let result = get_architecture(&projects, &args).unwrap();
        assert!(result.contains("authentication"));
        assert!(result.contains("JWT auth"));
        assert!(result.contains("src/auth.rs"));
    }

    #[test]
    fn test_get_architecture_case_insensitive() {
        let projects = create_test_projects();
        let args = json!({"project": "test-project", "concept": "AUTHENTICATION"});
        let result = get_architecture(&projects, &args).unwrap();
        assert!(result.contains("JWT auth"));
    }

    #[test]
    fn test_get_architecture_partial_match() {
        let projects = create_test_projects();
        let args = json!({"project": "test-project", "concept": "auth"});
        let result = get_architecture(&projects, &args).unwrap();
        assert!(result.contains("JWT auth"));
    }

    #[test]
    fn test_get_related_files() {
        let projects = create_test_projects();
        let args = json!({"project": "test-project", "query": "auth"});
        let result = get_related_files(&projects, &args).unwrap();
        assert!(result.contains("authentication"));
        assert!(result.contains("src/auth.rs"));
    }

    #[test]
    fn test_get_conventions() {
        let projects = create_test_projects();
        let args = json!({"project": "test-project"});
        let result = get_conventions(&projects, &args).unwrap();
        assert!(result.contains("naming"));
        assert!(result.contains("async"));
    }

    #[test]
    fn test_get_conventions_filtered() {
        let projects = create_test_projects();
        let args = json!({"project": "test-project", "category": "gotchas"});
        let result = get_conventions(&projects, &args).unwrap();
        assert!(result.contains("async"));
        assert!(!result.contains("naming"));
    }

    #[test]
    fn test_get_docs() {
        let projects = create_test_projects();
        let args = json!({"project": "test-project"});
        let result = get_docs(&projects, &args).unwrap();
        assert!(result.contains("readme"));
        assert!(result.contains("Project readme"));
    }

    #[test]
    fn test_get_docs_specific() {
        let projects = create_test_projects();
        let args = json!({"project": "test-project", "topic": "readme"});
        let result = get_docs(&projects, &args).unwrap();
        assert!(result.contains("README.md"));
    }

    #[test]
    fn test_get_workspace_overview_no_workspace() {
        let projects = create_test_projects();
        let root = PathBuf::from("/workspace");
        let result = get_workspace_overview(&root, &None, &projects).unwrap();
        assert!(result.contains("Workspace Overview"));
        assert!(result.contains("test-project"));
    }

    #[test]
    fn test_get_workspace_overview_with_workspace() {
        let projects = create_test_projects();
        let root = PathBuf::from("/workspace");
        let workspace = Some(WorkspaceConfig {
            workspace: WorkspaceInfo {
                name: Some("My Workspace".to_string()),
                description: Some("A test workspace".to_string()),
            },
            conventions: HashMap::new(),
            gotchas: HashMap::new(),
        });
        let result = get_workspace_overview(&root, &workspace, &projects).unwrap();
        assert!(result.contains("My Workspace"));
        assert!(result.contains("A test workspace"));
    }

    #[test]
    fn test_get_workspace_conventions_none() {
        let args = json!({});
        let result = get_workspace_conventions(&None, &args);
        assert!(result.is_err());
    }

    #[test]
    fn test_tools_list_contains_all_tools() {
        let list = tools_list();
        let tools = list["tools"].as_array().unwrap();
        
        let tool_names: Vec<&str> = tools
            .iter()
            .map(|t| t["name"].as_str().unwrap())
            .collect();
        
        assert!(tool_names.contains(&"list_projects"));
        assert!(tool_names.contains(&"get_project_info"));
        assert!(tool_names.contains(&"get_commands"));
        assert!(tool_names.contains(&"get_architecture"));
        assert!(tool_names.contains(&"get_related_files"));
        assert!(tool_names.contains(&"list_prompts"));
        assert!(tool_names.contains(&"get_prompt"));
        assert!(tool_names.contains(&"get_conventions"));
        assert!(tool_names.contains(&"get_docs"));
        assert!(tool_names.contains(&"get_workspace_overview"));
        assert!(tool_names.contains(&"get_workspace_conventions"));
    }
}
