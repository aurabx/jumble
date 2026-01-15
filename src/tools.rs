//! MCP tool implementations.

use crate::config::{
    Concept, ProjectConfig, ProjectConventions, ProjectDocs, ProjectSkills, WorkspaceConfig,
};
use crate::format::{
    format_api, format_commands, format_concept, format_dependencies, format_entry_points,
    format_related_projects,
};
use crate::memory::MemoryDatabase;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;

/// Type alias for project data stored in the server
pub type ProjectData = (
    PathBuf,
    ProjectConfig,
    ProjectSkills,
    ProjectConventions,
    ProjectDocs,
    MemoryDatabase,
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
                "name": "list_skills",
                "description": "Lists available task-specific skills for a project. Skills provide focused context for specific tasks like adding endpoints, debugging, etc.",
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
                "name": "get_skill",
                "description": "Retrieves a task-specific skill containing focused context and instructions for a particular task.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "project": {
                            "type": "string",
                            "description": "The project name"
                        },
                        "topic": {
                            "type": "string",
                            "description": "The skill topic (e.g., 'add-endpoint', 'debug-auth')"
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
            },
            {
                "name": "store_memory",
                "description": "Stores a memory entry (key-value pair) for a project. AI agents can use this to persist learned information, preferences, or context over time.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "project": {
                            "type": "string",
                            "description": "The project name"
                        },
                        "key": {
                            "type": "string",
                            "description": "The memory key (identifier)"
                        },
                        "value": {
                            "type": "string",
                            "description": "The memory value to store"
                        },
                        "source": {
                            "type": "string",
                            "description": "Optional: identifier for the agent/tool storing this memory"
                        }
                    },
                    "required": ["project", "key", "value"]
                }
            },
            {
                "name": "get_memory",
                "description": "Retrieves a specific memory entry by key for a project.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "project": {
                            "type": "string",
                            "description": "The project name"
                        },
                        "key": {
                            "type": "string",
                            "description": "The memory key to retrieve"
                        }
                    },
                    "required": ["project", "key"]
                }
            },
            {
                "name": "list_memories",
                "description": "Lists all stored memories for a project, optionally filtered by a key pattern.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "project": {
                            "type": "string",
                            "description": "The project name"
                        },
                        "pattern": {
                            "type": "string",
                            "description": "Optional: filter keys by this substring (case-insensitive)"
                        }
                    },
                    "required": ["project"]
                }
            },
            {
                "name": "search_memories",
                "description": "Searches memory keys and values for a query string (case-insensitive substring match).",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "project": {
                            "type": "string",
                            "description": "The project name"
                        },
                        "query": {
                            "type": "string",
                            "description": "Search query to match against keys and values"
                        }
                    },
                    "required": ["project", "query"]
                }
            },
            {
                "name": "delete_memory",
                "description": "Deletes a specific memory entry by key for a project.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "project": {
                            "type": "string",
                            "description": "The project name"
                        },
                        "key": {
                            "type": "string",
                            "description": "The memory key to delete"
                        }
                    },
                    "required": ["project", "key"]
                }
            },
            {
                "name": "clear_memories",
                "description": "Clears all memories for a project, optionally filtered by pattern or age. Use with caution!",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "project": {
                            "type": "string",
                            "description": "The project name"
                        },
                        "pattern": {
                            "type": "string",
                            "description": "Optional: only delete memories with keys matching this pattern (case-insensitive)"
                        },
                        "confirm": {
                            "type": "boolean",
                            "description": "Must be set to true to confirm deletion"
                        }
                    },
                    "required": ["project", "confirm"]
                }
            },
            {
                "name": "reload_workspace",
                "description": "Reloads workspace and project metadata from disk. Use this after editing .jumble files to pick up changes without restarting the server.",
                "inputSchema": {
                    "type": "object",
                    "properties": {},
                    "required": []
                }
            },
            {
                "name": "get_jumble_authoring_prompt",
                "description": "Returns a canonical prompt and guidance for creating .jumble context files (project, workspace, conventions, docs) in any project.",
                "inputSchema": {
                    "type": "object",
                    "properties": {},
                    "required": []
                }
            },
            {
                "name": "init_project",
                "description": "Initializes a new jumble project by creating the necessary directories and configuration files. Creates .jumble, .ai, docs directories and initializes project.toml, constitution.md, AGENTS.md, and .gitignore.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "directory": {
                            "type": "string",
                            "description": "The directory path to initialize the project in. Supports both absolute and relative paths."
                        }
                    },
                    "required": ["directory"]
                }
            }
        ]
    })
}

pub fn get_jumble_authoring_prompt() -> Result<String, String> {
    let prompt = r#"# Jumble authoring prompt

Use this prompt with an AI assistant to create Jumble context files for a project or workspace.

## Full prompt

```
Create jumble context for this project.

Read the AUTHORING.md guide at https://github.com/velvet-tiger/jumble/blob/main/AUTHORING.md, then examine this project's structure to create:

1. `.jumble/project.toml` (required)
   - Extract name, description, language from manifest files
   - Identify build/test/lint commands
   - Map 3–5 architectural concepts to their files
   - Note upstream/downstream project relationships

2. `.jumble/conventions.toml`
   - Capture coding patterns to follow (look at existing code)
   - Document gotchas and non-obvious behaviors
   - Check for constitution.md, CONTRIBUTING.md, or similar guides

3. `.jumble/docs.toml`
   - Index the docs/ directory if it exists
   - Write one-line summaries that help find the right doc

Focus on what helps an AI understand this codebase quickly. Don't over-document:
- 3–5 concepts
- 5–7 conventions/gotchas
- Index only human-written docs, not generated API docs
```

## Minimal prompt

```
Create jumble context for this project following the guide at https://github.com/velvet-tiger/jumble/blob/main/AUTHORING.md
```

## Workspace-level usage

For monorepos or multi-project workspaces, you can ask the AI to:

- Create `.jumble/workspace.toml` at the workspace root with:
  - Workspace name and description
  - Cross-project conventions (coding standards, tooling)
  - Common gotchas that span multiple projects
- Then, for each important project, create `.jumble/project.toml` with:
  - Project metadata and commands
  - Key concepts mapped to files
  - Upstream/downstream relationships to other workspace projects

Start with the most important projects. Use `related_projects` to show how they connect.
"#;

    Ok(prompt.to_string())
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
    for (name, (path, config, _skills, _conventions, _docs, _memory)) in projects {
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

    let (path, config, _skills, _conventions, _docs, _memory) = projects
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
            output.push_str(&format!(
                "**Description:** {}\n",
                config.project.description
            ));
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

    let (_, config, _, _, _, _) = projects
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

    let (path, config, _, _, _, _) = projects
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

    let (path, config, _, _, _, _) = projects
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

pub fn list_skills(
    projects: &HashMap<String, ProjectData>,
    args: &Value,
) -> Result<String, String> {
    let project_name = args
        .get("project")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'project' argument")?;

    let (_, _, skills, _, _, _) = projects
        .get(project_name)
        .ok_or_else(|| format!("Project '{}' not found", project_name))?;

    if skills.skills.is_empty() {
        return Ok(format!(
            "No skills found for '{}'. Create .jumble/skills/*.md files to add task-specific context.",
            project_name
        ));
    }

    let mut output = format!("Available skills for '{}':\n\n", project_name);

    // Include any available frontmatter description or, as a fallback, the first
    // line of the cached preview. This makes skill listings more informative
    // and exercises the cached metadata so it is not considered dead code.
    for (name, info) in &skills.skills {
        let mut line = format!("- {}", name);

        if let Some(fm) = &info.frontmatter {
            if let Some(desc) = &fm.description {
                if !desc.is_empty() {
                    line.push_str(&format!(": {}", desc));
                    output.push_str(&line);
                    output.push('\n');
                    continue;
                }
            }
        }

        let first_preview_line = info.preview.lines().next().unwrap_or("").trim();
        if !first_preview_line.is_empty() {
            line.push_str(&format!(": {}", first_preview_line));
        }

        output.push_str(&line);
        output.push('\n');
    }

    output.push_str("\nUse get_skill(project, topic) to retrieve a specific skill.");
    Ok(output)
}

pub fn get_skill(projects: &HashMap<String, ProjectData>, args: &Value) -> Result<String, String> {
    let project_name = args
        .get("project")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'project' argument")?;

    let topic = args
        .get("topic")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'topic' argument")?;

    let (_, _, skills, _, _, _) = projects
        .get(project_name)
        .ok_or_else(|| format!("Project '{}' not found", project_name))?;

    let skill_info = skills.skills.get(topic).ok_or_else(|| {
        let available: Vec<&str> = skills.skills.keys().map(|s| s.as_str()).collect();
        if available.is_empty() {
            format!("No skills found for '{}'", project_name)
        } else {
            format!(
                "Skill '{}' not found. Available: {}",
                topic,
                available.join(", ")
            )
        }
    })?;

    // Read the main skill file
    let skill_content = std::fs::read_to_string(&skill_info.path)
        .map_err(|e| format!("Failed to read skill: {}", e))?;

    // If this skill has a directory with companion files, include them
    if let Some(skill_dir) = &skill_info.skill_dir {
        let companions = discover_companion_files(skill_dir);
        if !companions.is_empty() {
            return Ok(format_skill_with_companions(&skill_content, &companions));
        }
    }

    Ok(skill_content)
}

/// Companion file entry discovered in a skill directory
#[derive(Debug)]
struct CompanionFile {
    relative_path: String,
    is_dir: bool,
}

/// Discover companion files and directories in a skill folder.
/// Looks for common subdirectories: scripts/, references/, docs/, assets/, examples/
fn discover_companion_files(skill_dir: &std::path::Path) -> Vec<CompanionFile> {
    let mut companions = Vec::new();

    // Common companion directory names for Claude/Codex skills
    let known_dirs = [
        "scripts",
        "references",
        "docs",
        "assets",
        "examples",
        "templates",
    ];

    for dir_name in &known_dirs {
        let dir_path = skill_dir.join(dir_name);
        if dir_path.is_dir() {
            // Add the directory itself
            companions.push(CompanionFile {
                relative_path: dir_name.to_string(),
                is_dir: true,
            });

            // List files in the directory (non-recursive for now)
            if let Ok(entries) = std::fs::read_dir(&dir_path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    if let Ok(file_type) = entry.file_type() {
                        if file_type.is_file() {
                            if let Some(file_name) = entry.file_name().to_str() {
                                companions.push(CompanionFile {
                                    relative_path: format!("{}/{}", dir_name, file_name),
                                    is_dir: false,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    companions
}

/// Format skill content with companion files listed at the end
fn format_skill_with_companions(skill_content: &str, companions: &[CompanionFile]) -> String {
    let mut output = String::from(skill_content);

    // Add companion files section
    output.push_str("\n\n---\n\n");
    output.push_str("## Companion Resources\n\n");
    output.push_str("This skill includes additional resources:\n\n");

    // Group by directory
    let mut current_dir: Option<String> = None;
    for companion in companions {
        if companion.is_dir {
            current_dir = Some(companion.relative_path.clone());
            output.push_str(&format!("\n### {}\n", companion.relative_path));
        } else {
            // Extract directory and filename
            if let Some(slash_pos) = companion.relative_path.rfind('/') {
                let dir = &companion.relative_path[..slash_pos];
                let file = &companion.relative_path[slash_pos + 1..];

                if current_dir.as_deref() == Some(dir) {
                    output.push_str(&format!("- `{}`\n", file));
                } else {
                    output.push_str(&format!("- `{}`\n", companion.relative_path));
                }
            } else {
                output.push_str(&format!("- `{}`\n", companion.relative_path));
            }
        }
    }

    output
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

    let (_, _, _, conventions, _, _) = projects
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

    let (path, _, _, _, docs, _) = projects
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
                format!("Doc '{}' not found. Available: {}", t, available.join(", "))
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
        let (_, config, _, _, _, _) = projects.get(*name).unwrap();
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
        let (_, config, _, _, _, _) = projects.get(*name).unwrap();
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

// ============================================================================
// Memory Tool Implementations
// ============================================================================

pub fn store_memory(
    projects: &HashMap<String, ProjectData>,
    args: &Value,
) -> Result<String, String> {
    let project_name = args
        .get("project")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'project' argument")?;

    let key = args
        .get("key")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'key' argument")?;

    let value = args
        .get("value")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'value' argument")?;

    let source = args.get("source").and_then(|v| v.as_str());

    let (_, _, _, _, _, memory_db) = projects
        .get(project_name)
        .ok_or_else(|| format!("Project '{}' not found", project_name))?;

    // Create memory entry
    let entry = crate::memory::MemoryEntry {
        value: value.to_string(),
        timestamp: crate::memory::current_timestamp(),
        source: source.map(|s| s.to_string()),
    };

    // Store in database
    memory_db
        .write(|db| {
            db.insert(key.to_string(), entry);
        })
        .map_err(|e| format!("Failed to write to memory database: {}", e))?;

    memory_db
        .save()
        .map_err(|e| format!("Failed to save memory database: {}", e))?;

    Ok(format!(
        "Memory stored: key='{}' for project '{}'",
        key, project_name
    ))
}

pub fn get_memory(projects: &HashMap<String, ProjectData>, args: &Value) -> Result<String, String> {
    let project_name = args
        .get("project")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'project' argument")?;

    let key = args
        .get("key")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'key' argument")?;

    let (_, _, _, _, _, memory_db) = projects
        .get(project_name)
        .ok_or_else(|| format!("Project '{}' not found", project_name))?;

    // Read from database
    let result = memory_db
        .read(|db| {
            db.get(key)
                .map(|entry| {
                    let mut output = format!("# Memory: {}\n\n", key);
                    output.push_str(&format!("**Value:** {}\n", entry.value));
                    output.push_str(&format!("**Timestamp:** {}\n", entry.timestamp));
                    if let Some(src) = &entry.source {
                        output.push_str(&format!("**Source:** {}\n", src));
                    }
                    output
                })
                .ok_or_else(|| format!("Memory key '{}' not found", key))
        })
        .map_err(|e| format!("Failed to read from memory database: {}", e))?;

    result
}

pub fn list_memories(
    projects: &HashMap<String, ProjectData>,
    args: &Value,
) -> Result<String, String> {
    let project_name = args
        .get("project")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'project' argument")?;

    let pattern = args.get("pattern").and_then(|v| v.as_str());

    let (_, _, _, _, _, memory_db) = projects
        .get(project_name)
        .ok_or_else(|| format!("Project '{}' not found", project_name))?;

    // Read from database
    let result = memory_db
        .read(|db| {
            if db.is_empty() {
                return Ok(format!("No memories stored for project '{}'", project_name));
            }

            let mut keys: Vec<&String> = db.keys().collect();
            keys.sort();

            // Filter by pattern if provided
            let filtered_keys: Vec<&String> = if let Some(pat) = pattern {
                let pat_lower = pat.to_lowercase();
                keys.into_iter()
                    .filter(|k| k.to_lowercase().contains(&pat_lower))
                    .collect()
            } else {
                keys
            };

            if filtered_keys.is_empty() {
                return Ok(format!(
                    "No memories matching pattern '{}' for project '{}'",
                    pattern.unwrap_or(""),
                    project_name
                ));
            }

            let mut output = format!("# Memories for '{}'\n\n", project_name);
            if let Some(pat) = pattern {
                output.push_str(&format!("Filtered by: {}\n\n", pat));
            }

            for key in filtered_keys {
                if let Some(entry) = db.get(key) {
                    output.push_str(&format!("- **{}**\n", key));
                    output.push_str(&format!("  Timestamp: {}\n", entry.timestamp));
                    if let Some(src) = &entry.source {
                        output.push_str(&format!("  Source: {}\n", src));
                    }
                    // Preview first 100 chars of value
                    let preview = if entry.value.len() > 100 {
                        format!("{}...", &entry.value[..100])
                    } else {
                        entry.value.clone()
                    };
                    output.push_str(&format!("  Preview: {}\n", preview));
                }
            }

            Ok(output)
        })
        .map_err(|e| format!("Failed to read from memory database: {}", e))?;

    result
}

pub fn search_memories(
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

    let (_, _, _, _, _, memory_db) = projects
        .get(project_name)
        .ok_or_else(|| format!("Project '{}' not found", project_name))?;

    // Read from database
    let result = memory_db
        .read(|db| {
            if db.is_empty() {
                return Ok(format!("No memories stored for project '{}'", project_name));
            }

            let query_lower = query.to_lowercase();
            let mut matches: Vec<(&String, &crate::memory::MemoryEntry)> = db
                .iter()
                .filter(|(k, v)| {
                    k.to_lowercase().contains(&query_lower)
                        || v.value.to_lowercase().contains(&query_lower)
                })
                .collect();

            if matches.is_empty() {
                return Ok(format!(
                    "No memories matching query '{}' for project '{}'",
                    query, project_name
                ));
            }

            // Sort by key for consistent output
            matches.sort_by_key(|(k, _)| *k);

            let mut output = format!("# Search results for '{}' in '{}'\n\n", query, project_name);
            output.push_str(&format!("Found {} match(es)\n\n", matches.len()));

            for (key, entry) in matches {
                output.push_str(&format!("## {}\n", key));
                output.push_str(&format!("**Value:** {}\n", entry.value));
                output.push_str(&format!("**Timestamp:** {}\n", entry.timestamp));
                if let Some(src) = &entry.source {
                    output.push_str(&format!("**Source:** {}\n", src));
                }
                output.push('\n');
            }

            Ok(output)
        })
        .map_err(|e| format!("Failed to read from memory database: {}", e))?;

    result
}

pub fn delete_memory(
    projects: &HashMap<String, ProjectData>,
    args: &Value,
) -> Result<String, String> {
    let project_name = args
        .get("project")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'project' argument")?;

    let key = args
        .get("key")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'key' argument")?;

    let (_, _, _, _, _, memory_db) = projects
        .get(project_name)
        .ok_or_else(|| format!("Project '{}' not found", project_name))?;

    // Delete from database
    let deleted = memory_db
        .write(|db| db.remove(key).is_some())
        .map_err(|e| format!("Failed to write to memory database: {}", e))?;

    if !deleted {
        return Err(format!("Memory key '{}' not found", key));
    }

    memory_db
        .save()
        .map_err(|e| format!("Failed to save memory database: {}", e))?;

    Ok(format!(
        "Memory deleted: key='{}' for project '{}'",
        key, project_name
    ))
}

pub fn clear_memories(
    projects: &HashMap<String, ProjectData>,
    args: &Value,
) -> Result<String, String> {
    let project_name = args
        .get("project")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'project' argument")?;

    let confirm = args
        .get("confirm")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if !confirm {
        return Err("Deletion not confirmed. Set 'confirm' to true to proceed.".to_string());
    }

    let pattern = args.get("pattern").and_then(|v| v.as_str());

    let (_, _, _, _, _, memory_db) = projects
        .get(project_name)
        .ok_or_else(|| format!("Project '{}' not found", project_name))?;

    // Delete from database
    let deleted_count = memory_db
        .write(|db| {
            if let Some(pat) = pattern {
                let pat_lower = pat.to_lowercase();
                let keys_to_delete: Vec<String> = db
                    .keys()
                    .filter(|k| k.to_lowercase().contains(&pat_lower))
                    .cloned()
                    .collect();

                let count = keys_to_delete.len();
                for key in keys_to_delete {
                    db.remove(&key);
                }
                count
            } else {
                let count = db.len();
                db.clear();
                count
            }
        })
        .map_err(|e| format!("Failed to write to memory database: {}", e))?;

    memory_db
        .save()
        .map_err(|e| format!("Failed to save memory database: {}", e))?;

    if let Some(pat) = pattern {
        Ok(format!(
            "Cleared {} memor{} matching pattern '{}' for project '{}'",
            deleted_count,
            if deleted_count == 1 { "y" } else { "ies" },
            pat,
            project_name
        ))
    } else {
        Ok(format!(
            "Cleared all {} memor{} for project '{}'",
            deleted_count,
            if deleted_count == 1 { "y" } else { "ies" },
            project_name
        ))
    }
}

pub fn init_project(_workspace_root: &std::path::PathBuf, args: &Value) -> Result<String, String> {
    // Get the target directory from arguments (required)
    let dir_str = args
        .get("directory")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'directory' argument")?;
    
    let target_dir = std::path::PathBuf::from(dir_str);
    
    // Ensure the directory exists or can be created
    if !target_dir.exists() {
        std::fs::create_dir_all(&target_dir)
            .map_err(|e| format!("Failed to create directory '{}': {}", target_dir.display(), e))?;
    }
    
    // Use the setup module's init function
    match crate::setup::setup_init(&target_dir) {
        Ok(()) => Ok(format!("Project initialized successfully in {}.", target_dir.display())),
        Err(e) => Err(format!("Failed to initialize project: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::*;
    use crate::memory;
    use std::path::PathBuf;
    use tempfile::TempDir;

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

        let skills = ProjectSkills::default();
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

        // Create a temporary memory database for testing
        let temp_dir = TempDir::new().unwrap();
        let test_path = temp_dir.path().to_path_buf();
        let memory_db = memory::open_or_create_memory_db(&test_path).unwrap();

        (
            "test-project".to_string(),
            (
                test_path.clone(),
                config,
                skills,
                conventions,
                docs,
                memory_db,
            ),
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

        let tool_names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();

        assert!(tool_names.contains(&"list_projects"));
        assert!(tool_names.contains(&"get_project_info"));
        assert!(tool_names.contains(&"get_commands"));
        assert!(tool_names.contains(&"get_architecture"));
        assert!(tool_names.contains(&"get_related_files"));
        assert!(tool_names.contains(&"list_skills"));
        assert!(tool_names.contains(&"get_skill"));
        assert!(tool_names.contains(&"get_conventions"));
        assert!(tool_names.contains(&"get_docs"));
        assert!(tool_names.contains(&"get_workspace_overview"));
        assert!(tool_names.contains(&"get_workspace_conventions"));
        assert!(tool_names.contains(&"reload_workspace"));
        assert!(tool_names.contains(&"get_jumble_authoring_prompt"));
    }

    #[test]
    fn test_discover_companion_files_empty_directory() {
        // Create a temporary skill directory with no companion files
        let tmp_dir = std::env::temp_dir().join("jumble_test_empty_skill");
        std::fs::create_dir_all(&tmp_dir).unwrap();

        let companions = discover_companion_files(&tmp_dir);

        // Clean up
        let _ = std::fs::remove_dir_all(&tmp_dir);

        assert!(companions.is_empty());
    }

    #[test]
    fn test_discover_companion_files_with_scripts() {
        // Create a temporary skill directory with scripts/ subdirectory
        let tmp_dir = std::env::temp_dir().join("jumble_test_skill_with_scripts");
        let scripts_dir = tmp_dir.join("scripts");
        std::fs::create_dir_all(&scripts_dir).unwrap();
        std::fs::write(scripts_dir.join("helper.sh"), "#!/bin/bash\necho test").unwrap();
        std::fs::write(scripts_dir.join("setup.py"), "print('setup')").unwrap();

        let companions = discover_companion_files(&tmp_dir);

        // Clean up
        let _ = std::fs::remove_dir_all(&tmp_dir);

        // Should find the scripts directory and the two files
        assert!(!companions.is_empty());
        assert!(companions
            .iter()
            .any(|c| c.relative_path == "scripts" && c.is_dir));
        assert!(companions
            .iter()
            .any(|c| c.relative_path.contains("helper.sh") && !c.is_dir));
        assert!(companions
            .iter()
            .any(|c| c.relative_path.contains("setup.py") && !c.is_dir));
    }

    #[test]
    fn test_discover_companion_files_multiple_dirs() {
        // Create a temporary skill directory with multiple companion directories
        let tmp_dir = std::env::temp_dir().join("jumble_test_skill_multi");
        let scripts_dir = tmp_dir.join("scripts");
        let refs_dir = tmp_dir.join("references");
        let assets_dir = tmp_dir.join("assets");

        std::fs::create_dir_all(&scripts_dir).unwrap();
        std::fs::create_dir_all(&refs_dir).unwrap();
        std::fs::create_dir_all(&assets_dir).unwrap();

        std::fs::write(scripts_dir.join("build.sh"), "#!/bin/bash").unwrap();
        std::fs::write(refs_dir.join("api-docs.md"), "# API").unwrap();
        std::fs::write(assets_dir.join("template.json"), "{}").unwrap();

        let companions = discover_companion_files(&tmp_dir);

        // Clean up
        let _ = std::fs::remove_dir_all(&tmp_dir);

        // Should find all three directories
        assert!(companions
            .iter()
            .any(|c| c.relative_path == "scripts" && c.is_dir));
        assert!(companions
            .iter()
            .any(|c| c.relative_path == "references" && c.is_dir));
        assert!(companions
            .iter()
            .any(|c| c.relative_path == "assets" && c.is_dir));

        // And all three files
        assert!(companions
            .iter()
            .any(|c| c.relative_path.contains("build.sh")));
        assert!(companions
            .iter()
            .any(|c| c.relative_path.contains("api-docs.md")));
        assert!(companions
            .iter()
            .any(|c| c.relative_path.contains("template.json")));
    }

    #[test]
    fn test_format_skill_with_companions() {
        let skill_content = "# My Skill\n\nThis is a test skill.";
        let companions = vec![
            CompanionFile {
                relative_path: "scripts".to_string(),
                is_dir: true,
            },
            CompanionFile {
                relative_path: "scripts/helper.sh".to_string(),
                is_dir: false,
            },
            CompanionFile {
                relative_path: "references".to_string(),
                is_dir: true,
            },
            CompanionFile {
                relative_path: "references/guide.md".to_string(),
                is_dir: false,
            },
        ];

        let result = format_skill_with_companions(skill_content, &companions);

        // Should contain original content
        assert!(result.contains("# My Skill"));
        assert!(result.contains("This is a test skill."));

        // Should contain companion resources section
        assert!(result.contains("## Companion Resources"));
        assert!(result.contains("### scripts"));
        assert!(result.contains("`helper.sh`"));
        assert!(result.contains("### references"));
        assert!(result.contains("`guide.md`"));
    }
}
