//! Setup commands for configuring AI agents to use jumble effectively

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Initialize a new jumble project by creating necessary directories and config files
pub fn setup_init(workspace_root: &Path) -> Result<()> {
    // Create .jumble directory
    let jumble_dir = workspace_root.join(".jumble");
    if jumble_dir.exists() {
        println!("✓ .jumble directory already exists");
    } else {
        fs::create_dir_all(&jumble_dir).context("Failed to create .jumble directory")?;
        println!("✓ Created .jumble directory");
    }

    // Create .jumble/project.toml if it doesn't exist
    let project_toml = jumble_dir.join("project.toml");
    if project_toml.exists() {
        println!("✓ .jumble/project.toml already exists");
    } else {
        let default_project = r#"[project]
name = "my-project"
description = "A brief description of your project"

# [commands]
# build = "cargo build"
# test = "cargo test"
# lint = "cargo clippy"

# [entry_points]
# main = "src/main.rs"
"#;
        fs::write(&project_toml, default_project)
            .context("Failed to create .jumble/project.toml")?;
        println!("✓ Created .jumble/project.toml (edit to configure)");
    }

    // Create .ai directory
    let ai_dir = workspace_root.join(".ai");
    if ai_dir.exists() {
        println!("✓ .ai directory already exists");
    } else {
        fs::create_dir_all(&ai_dir).context("Failed to create .ai directory")?;
        println!("✓ Created .ai directory");
    }

    // Create .ai/constitution.md if it doesn't exist
    let constitution = ai_dir.join("constitution.md");
    if constitution.exists() {
        println!("✓ .ai/constitution.md already exists");
    } else {
        fs::write(&constitution, "").context("Failed to create .ai/constitution.md")?;
        println!("✓ Created .ai/constitution.md (blank for now)");
    }

    // Create docs directory
    let docs_dir = workspace_root.join("docs");
    if docs_dir.exists() {
        println!("✓ docs directory already exists");
    } else {
        fs::create_dir_all(&docs_dir).context("Failed to create docs directory")?;
        println!("✓ Created docs directory");
    }

    // Create AGENTS.md
    let agents_md = workspace_root.join("AGENTS.md");
    if agents_md.exists() {
        println!("✓ AGENTS.md already exists");
    } else {
        let agents_content = r#"# Using Jumble in This Project

This project uses Jumble to provide AI agents with structured context about the codebase.

## Getting Started

1. **Always start by calling `get_workspace_overview()`** from the Jumble MCP server to understand the project structure.

2. **Check the project constitution** in `.ai/constitution.md` for any project-specific guidelines and rules that should be followed.

3. **Use the Jumble tools** to access project metadata before making changes.

## When to Use Jumble Tools

### Before suggesting commands
- Call `get_commands(project, type)` to get exact build/test/lint/run commands
- Never guess commands when Jumble can provide them

### Before making architectural changes
- Call `get_architecture(project, concept)` to understand existing patterns
- Use `get_related_files(project, query)` to find related code

### Before writing new code
- Call `get_conventions(project)` for project-specific patterns
- Call `get_workspace_conventions()` for workspace-wide standards
- Review both conventions AND gotchas

### Before searching for documentation
- Call `get_docs(project)` to see available documentation
- Use topic names to get specific doc paths

## Project Guidelines

See `.ai/constitution.md` for project-specific guidelines, conventions, and any special instructions for AI agents working on this codebase.

## Available Jumble Tools

- `list_projects` - List all projects in workspace
- `get_workspace_overview` - Workspace structure and dependencies
- `get_workspace_conventions` - Workspace-level conventions/gotchas
- `get_project_info` - Project metadata and structure
- `get_commands` - Build/test/lint/run commands
- `get_architecture` - Architectural concepts and files
- `get_related_files` - Find files by concept
- `get_conventions` - Project conventions and gotchas
- `get_docs` - Documentation index
- `list_skills` / `get_skill` - Task-specific guidance
"#;
        fs::write(&agents_md, agents_content)
            .context("Failed to create AGENTS.md")?;
        println!("✓ Created AGENTS.md");
    }

    // Create .gitignore if it doesn't exist (empty by default)
    let gitignore = workspace_root.join(".gitignore");
    if gitignore.exists() {
        println!("✓ .gitignore already exists");
    } else {
        fs::write(&gitignore, "").context("Failed to create .gitignore")?;
        println!("✓ Created .gitignore");
    }

    println!();
    println!("✨ Project initialized!");
    println!();
    println!("Next steps:");
    println!("1. Read AGENTS.md for guidance on using Jumble");
    println!("2. Add your project guidelines to .ai/constitution.md");
    println!("3. Edit .jumble/project.toml to configure your project");
    println!("4. Add project documentation to the docs/ directory");
    println!("5. Commit these files to version control");

    Ok(())
}

const JUMBLE_SECTION: &str = r#"## Using Jumble for Project Context

ALWAYS start workspace exploration by calling `get_workspace_overview()` from the Jumble MCP server to understand the workspace structure, available projects, and their relationships.

### When to Use Jumble Tools

**Before suggesting commands:**
- Call `get_commands(project, type)` to get exact build/test/lint/run commands
- Never guess commands when jumble can provide them

**Before making architectural changes:**
- Call `get_architecture(project, concept)` to understand existing patterns
- Use `get_related_files(project, query)` to find related code

**Before writing new code:**
- Call `get_conventions(project)` for project-specific patterns
- Call `get_workspace_conventions()` for workspace-wide standards
- Review both conventions AND gotchas

**Before searching for documentation:**
- Call `get_docs(project)` to see available documentation
- Use topic names to get specific doc paths

**For specific tasks:**
- Call `list_skills(project)` to see available task-specific guidance
- Use `get_skill(project, topic)` for focused instructions

### Handling Missing Context

If jumble returns "No projects found":
1. Call `get_jumble_authoring_prompt()` to get the creation prompt
2. Offer to create `.jumble/project.toml` for the current project
3. Follow the AUTHORING.md guide

### Workflow

1. **Enter workspace** → `get_workspace_overview()`
2. **Working on a project** → `get_project_info(project)`
3. **Making changes** → Check conventions, architecture, skills
4. **Writing code** → Follow conventions, avoid gotchas
5. **Running commands** → Use `get_commands(project, type)`
"#;

const JUMBLE_SECTION_MARKER: &str = "## Using Jumble for Project Context";

/// Setup Warp integration by creating/updating WARP.md
pub fn setup_warp(workspace_root: &Path, force: bool) -> Result<()> {
    let warp_md = workspace_root.join("WARP.md");

    if warp_md.exists() {
        let content = fs::read_to_string(&warp_md).context("Failed to read WARP.md")?;

        if content.contains(JUMBLE_SECTION_MARKER) {
            if !force {
                println!("✓ WARP.md already contains jumble rules");
                println!();
                println!("To update the jumble section, run with --force:");
                println!("  jumble setup warp --force");
                return Ok(());
            }

            // Replace existing section
            let updated = replace_jumble_section(&content)?;
            fs::write(&warp_md, updated).context("Failed to update WARP.md")?;
            println!("✓ Updated jumble rules in WARP.md");
        } else {
            // Append jumble section
            let mut updated = content;
            if !updated.ends_with('\n') {
                updated.push('\n');
            }
            updated.push('\n');
            updated.push_str(JUMBLE_SECTION);

            fs::write(&warp_md, updated).context("Failed to update WARP.md")?;
            println!("✓ Added jumble rules to existing WARP.md");
        }
    } else {
        // Create new WARP.md
        let content = format!(
            "# WARP.md\n\nThis file provides guidance to WARP (warp.dev) when working with code in this repository.\n\n{}",
            JUMBLE_SECTION
        );

        fs::write(&warp_md, content).context("Failed to create WARP.md")?;
        println!("✓ Created WARP.md with jumble rules");
    }

    // Check for .jumble directory
    let jumble_dir = workspace_root.join(".jumble");
    if !jumble_dir.exists() {
        println!();
        println!("⚠️  No .jumble directory found");
        println!("   Create .jumble/project.toml to provide project context");
        println!("   See: https://github.com/velvet-tiger/jumble/blob/main/AUTHORING.md");
    }

    println!();
    println!("Next steps:");
    println!("1. Ensure .jumble/project.toml exists (provides context to jumble)");
    println!("2. Verify jumble MCP server is configured in Warp:");
    println!("   - Open Warp settings → AI → MCP Servers");
    println!("   - Add jumble with: --root {}", workspace_root.display());
    println!("3. Restart Warp or reload the window to apply changes");
    println!("4. Commit WARP.md to version control");

    Ok(())
}

/// Replace the jumble section in existing WARP.md content
fn replace_jumble_section(content: &str) -> Result<String> {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();
    let mut in_jumble_section = false;

    for line in lines {
        if line.starts_with("## Using Jumble for Project Context") {
            in_jumble_section = true;
            continue;
        }

        if in_jumble_section {
            // Check if we've hit another section at same or higher level
            if line.starts_with("# ") || (line.starts_with("## ") && !line.contains("Using Jumble")) {
                in_jumble_section = false;
            }
        }

        if !in_jumble_section {
            result.push(line);
        }
    }

    // Find the best place to insert the updated section
    // Try to insert before the first H1 after any existing content
    let insert_pos = result
        .iter()
        .position(|&line| line.starts_with("# ") && !line.starts_with("# WARP"))
        .unwrap_or(result.len());

    // Add the new jumble section
    let jumble_lines: Vec<&str> = JUMBLE_SECTION.lines().collect();

    // Insert with proper spacing
    if insert_pos < result.len() {
        result.insert(insert_pos, "");
        for (i, line) in jumble_lines.iter().enumerate() {
            result.insert(insert_pos + i, line);
        }
        result.insert(insert_pos + jumble_lines.len(), "");
    } else {
        // Append to end
        result.push("");
        result.extend(jumble_lines);
    }

    Ok(result.join("\n"))
}

const USAGE_GUIDE: &str = r#"# Using Jumble for Project Context

Jumble provides queryable, on-demand project context to help you work more effectively.

## Getting Started

**Always start by calling `get_workspace_overview()`** to understand the workspace structure, available projects, and their relationships.

## When to Use Jumble Tools

### Before suggesting commands
- Call `get_commands(project, type)` to get exact build/test/lint/run commands
- Never guess commands when jumble can provide them

### Before making architectural changes
- Call `get_architecture(project, concept)` to understand existing patterns
- Use `get_related_files(project, query)` to find related code

### Before writing new code
- Call `get_conventions(project)` for project-specific patterns
- Call `get_workspace_conventions()` for workspace-wide standards
- Review both conventions AND gotchas

### Before searching for documentation
- Call `get_docs(project)` to see available documentation
- Use topic names to get specific doc paths

### For specific tasks
- Call `list_skills(project)` to see available task-specific guidance
- Use `get_skill(project, topic)` for focused instructions

## Handling Missing Context

If jumble returns "No projects found":
1. Call `get_jumble_authoring_prompt()` to get the creation prompt
2. Offer to create `.jumble/project.toml` for the current project
3. Follow the AUTHORING.md guide

## Workflow

1. **Enter workspace** → `get_workspace_overview()`
2. **Working on a project** → `get_project_info(project)`
3. **Making changes** → Check conventions, architecture, skills
4. **Writing code** → Follow conventions, avoid gotchas
5. **Running commands** → Use `get_commands(project, type)`

## Available Tools

- `list_projects` - List all projects in workspace
- `get_workspace_overview` - Workspace structure and dependencies
- `get_workspace_conventions` - Workspace-level conventions/gotchas
- `get_project_info` - Project metadata and structure
- `get_commands` - Build/test/lint/run commands
- `get_architecture` - Architectural concepts and files
- `get_related_files` - Find files by concept
- `get_conventions` - Project conventions and gotchas
- `get_docs` - Documentation index
- `list_skills` / `get_skill` - Task-specific guidance
"#;

/// Setup Claude Desktop integration
pub fn setup_claude(workspace_root: &Path, global: bool) -> Result<()> {
    let config_dir = if global {
        dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
            .join(".claude")
    } else {
        workspace_root.join(".claude")
    };

    fs::create_dir_all(&config_dir).context("Failed to create .claude directory")?;

    let guide_path = config_dir.join("jumble-usage.md");
    fs::write(&guide_path, USAGE_GUIDE).context("Failed to write usage guide")?;

    println!("✓ Created {}", guide_path.display());

    // Check MCP config
    let mcp_config = dirs::home_dir()
        .map(|h| h.join("Library/Application Support/Claude/claude_desktop_config.json"));

    if let Some(config_path) = mcp_config {
        if config_path.exists() {
            let content =
                fs::read_to_string(&config_path).context("Failed to read Claude config")?;

            if content.contains("\"jumble\"") {
                println!("✓ Jumble MCP server detected in Claude Desktop config");
            } else {
                println!();
                println!("⚠️  Jumble not found in Claude Desktop config");
                println!("   Add to {}:", config_path.display());
                println!();
                println!("   {{");
                println!("     \"mcpServers\": {{");
                println!("       \"jumble\": {{");
                let jumble_path = which::which("jumble")
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|_| "/path/to/jumble".to_string());
                println!("         \"command\": \"{}\",", jumble_path);
                println!(
                    "         \"args\": [\"--root\", \"{}\"]",
                    workspace_root.display()
                );
                println!("       }}");
                println!("     }}");
                println!("   }}");
                println!();
                println!("   Then restart Claude Desktop.");
            }
        } else {
            println!();
            println!("⚠️  Claude Desktop config not found");
            println!("   Expected: {}", config_path.display());
            println!("   Configure jumble in Claude Desktop settings.");
        }
    }

    print_common_next_steps(workspace_root, "Claude Desktop");
    Ok(())
}

/// Setup Cursor integration
pub fn setup_cursor(workspace_root: &Path, global: bool) -> Result<()> {
    let config_dir = if global {
        dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
            .join(".cursor")
    } else {
        workspace_root.join(".cursor")
    };

    fs::create_dir_all(&config_dir).context("Failed to create .cursor directory")?;

    let guide_path = config_dir.join("jumble-usage.md");
    fs::write(&guide_path, USAGE_GUIDE).context("Failed to write usage guide")?;

    println!("✓ Created {}", guide_path.display());

    // Check/create MCP config
    let mcp_config_path = config_dir.join("mcp.json");

    if mcp_config_path.exists() {
        let content =
            fs::read_to_string(&mcp_config_path).context("Failed to read Cursor MCP config")?;

        if content.contains("\"jumble\"") {
            println!(
                "✓ Jumble already configured in {}",
                mcp_config_path.display()
            );
        } else {
            println!();
            println!("⚠️  Jumble not found in Cursor MCP config");
            print_cursor_config_instructions(&mcp_config_path, workspace_root);
        }
    } else {
        println!();
        println!("📝 Creating Cursor MCP config...");
        print_cursor_config_instructions(&mcp_config_path, workspace_root);
    }

    print_common_next_steps(workspace_root, "Cursor");
    Ok(())
}

/// Setup Windsurf integration
pub fn setup_windsurf(workspace_root: &Path, global: bool) -> Result<()> {
    let config_dir = if global {
        dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
            .join(".codeium/windsurf")
    } else {
        workspace_root.join(".windsurf")
    };

    fs::create_dir_all(&config_dir).context("Failed to create windsurf config directory")?;

    let guide_path = config_dir.join("jumble-usage.md");
    fs::write(&guide_path, USAGE_GUIDE).context("Failed to write usage guide")?;

    println!("✓ Created {}", guide_path.display());

    // Check MCP config
    let mcp_config_path = dirs::home_dir().map(|h| h.join(".codeium/windsurf/mcp_config.json"));

    if let Some(config_path) = mcp_config_path {
        if config_path.exists() {
            let content =
                fs::read_to_string(&config_path).context("Failed to read Windsurf config")?;

            if content.contains("\"jumble\"") {
                println!("✓ Jumble MCP server detected in Windsurf config");
            } else {
                println!();
                println!("⚠️  Jumble not found in Windsurf config");
                print_windsurf_config_instructions(&config_path, workspace_root);
            }
        } else {
            println!();
            println!("⚠️  Windsurf config not found");
            println!("   Expected: {}", config_path.display());
            print_windsurf_config_instructions(&config_path, workspace_root);
        }
    }

    print_common_next_steps(workspace_root, "Windsurf");
    Ok(())
}

/// Setup Codex integration
pub fn setup_codex(workspace_root: &Path, global: bool) -> Result<()> {
    let config_dir = if global {
        dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
            .join(".codex")
    } else {
        workspace_root.join(".codex")
    };

    fs::create_dir_all(&config_dir).context("Failed to create .codex directory")?;

    let guide_path = config_dir.join("jumble-usage.md");
    fs::write(&guide_path, USAGE_GUIDE).context("Failed to write usage guide")?;

    println!("✓ Created {}", guide_path.display());

    // Check MCP config
    let config_path = dirs::home_dir().map(|h| h.join(".codex/config.toml"));

    if let Some(config_file) = config_path {
        if config_file.exists() {
            let content =
                fs::read_to_string(&config_file).context("Failed to read Codex config")?;

            if content.contains("[mcp_servers.jumble]") {
                println!("✓ Jumble MCP server detected in Codex config");
            } else {
                println!();
                println!("⚠️  Jumble not found in Codex config");
                print_codex_config_instructions(&config_file, workspace_root);
            }
        } else {
            println!();
            println!("⚠️  Codex config not found");
            println!("   Expected: {}", config_file.display());
            print_codex_config_instructions(&config_file, workspace_root);
        }
    }

    print_common_next_steps(workspace_root, "Codex");
    Ok(())
}

fn print_cursor_config_instructions(config_path: &Path, workspace_root: &Path) {
    println!("   Add to {}:", config_path.display());
    println!();
    println!("   {{");
    println!("     \"mcpServers\": {{");
    println!("       \"jumble\": {{");
    let jumble_path = which::which("jumble")
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "/path/to/jumble".to_string());
    println!("         \"command\": \"{}\",", jumble_path);
    println!(
        "         \"args\": [\"--root\", \"{}\"]",
        workspace_root.display()
    );
    println!("       }}");
    println!("     }}");
    println!("   }}");
}

fn print_windsurf_config_instructions(config_path: &Path, workspace_root: &Path) {
    println!("   Add to {}:", config_path.display());
    println!();
    println!("   {{");
    println!("     \"mcpServers\": {{");
    println!("       \"jumble\": {{");
    let jumble_path = which::which("jumble")
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "/path/to/jumble".to_string());
    println!("         \"command\": \"{}\",", jumble_path);
    println!(
        "         \"args\": [\"--root\", \"{}\"]",
        workspace_root.display()
    );
    println!("       }}");
    println!("     }}");
    println!("   }}");
    println!();
    println!("   Then restart Windsurf.");
}

fn print_codex_config_instructions(config_path: &Path, workspace_root: &Path) {
    println!("   Add to {}:", config_path.display());
    println!();
    println!("   [mcp_servers.jumble]");
    let jumble_path = which::which("jumble")
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "/path/to/jumble".to_string());
    println!("   command = \"{}\"", jumble_path);
    println!("   args = [\"--root\", \"{}\"]", workspace_root.display());
    println!();
    println!("   Or use the CLI:");
    println!(
        "   codex mcp add jumble -- {} --root {}",
        jumble_path,
        workspace_root.display()
    );
    println!();
    println!("   Then restart Codex.");
}

fn print_common_next_steps(workspace_root: &Path, agent_name: &str) {
    let jumble_dir = workspace_root.join(".jumble");
    if !jumble_dir.exists() {
        println!();
        println!("⚠️  No .jumble directory found");
        println!("   Create .jumble/project.toml to provide project context");
        println!("   See: https://github.com/velvet-tiger/jumble/blob/main/AUTHORING.md");
    }

    println!();
    println!("Next steps:");
    println!("1. Ensure .jumble/project.toml exists");
    println!(
        "2. Verify jumble MCP server is configured in {}",
        agent_name
    );
    println!("3. Restart {} to apply changes", agent_name);
    println!("4. Read the usage guide for best practices");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_setup_init_creates_all_directories_and_files() {
        let temp = TempDir::new().unwrap();
        let workspace = temp.path();

        setup_init(workspace).unwrap();

        // Check all directories exist
        assert!(workspace.join(".jumble").is_dir());
        assert!(workspace.join(".ai").is_dir());
        assert!(workspace.join("docs").is_dir());

        // Check all files exist
        assert!(workspace.join(".jumble/project.toml").is_file());
        assert!(workspace.join(".ai/constitution.md").is_file());
        assert!(workspace.join("AGENTS.md").is_file());
        assert!(workspace.join(".gitignore").is_file());

        // Check project.toml content
        let project_content = fs::read_to_string(workspace.join(".jumble/project.toml")).unwrap();
        assert!(project_content.contains("[project]"));
        assert!(project_content.contains("name = \"my-project\""));

        // Check constitution.md is blank
        let constitution_content = fs::read_to_string(workspace.join(".ai/constitution.md")).unwrap();
        assert_eq!(constitution_content, "");

        // Check AGENTS.md content
        let agents_content = fs::read_to_string(workspace.join("AGENTS.md")).unwrap();
        assert!(agents_content.contains("Using Jumble in This Project"));
        assert!(agents_content.contains(".ai/constitution.md"));
        assert!(agents_content.contains("get_workspace_overview"));

        // Check gitignore exists (empty by default)
        let gitignore_content = fs::read_to_string(workspace.join(".gitignore")).unwrap();
        assert_eq!(gitignore_content, "");
    }

    #[test]
    fn test_setup_init_is_idempotent() {
        let temp = TempDir::new().unwrap();
        let workspace = temp.path();

        // Run twice
        setup_init(workspace).unwrap();
        let first_project_content = fs::read_to_string(workspace.join(".jumble/project.toml")).unwrap();

        setup_init(workspace).unwrap();
        let second_project_content = fs::read_to_string(workspace.join(".jumble/project.toml")).unwrap();

        // Content should be identical
        assert_eq!(first_project_content, second_project_content);
    }

    #[test]
    fn test_setup_init_preserves_existing_gitignore() {
        let temp = TempDir::new().unwrap();
        let workspace = temp.path();

        // Create existing .gitignore
        let gitignore_path = workspace.join(".gitignore");
        fs::write(&gitignore_path, "*.log\n*.tmp\n").unwrap();

        setup_init(workspace).unwrap();

        let gitignore_content = fs::read_to_string(&gitignore_path).unwrap();
        // Check original entries are preserved and unchanged
        assert_eq!(gitignore_content, "*.log\n*.tmp\n");
    }

    #[test]
    fn test_setup_warp_creates_new_file() {
        let temp = TempDir::new().unwrap();
        let workspace = temp.path();

        setup_warp(workspace, false).unwrap();

        let warp_md = workspace.join("WARP.md");
        assert!(warp_md.exists());

        let content = fs::read_to_string(warp_md).unwrap();
        assert!(content.contains("## Using Jumble for Project Context"));
        assert!(content.contains("get_workspace_overview()"));
    }

    #[test]
    fn test_setup_warp_appends_to_existing() {
        let temp = TempDir::new().unwrap();
        let workspace = temp.path();
        let warp_md = workspace.join("WARP.md");

        // Create existing WARP.md
        fs::write(
            &warp_md,
            "# WARP.md\n\n## Existing Section\n\nSome content.\n",
        )
        .unwrap();

        setup_warp(workspace, false).unwrap();

        let content = fs::read_to_string(warp_md).unwrap();
        assert!(content.contains("## Existing Section"));
        assert!(content.contains("## Using Jumble for Project Context"));
    }

    #[test]
    fn test_setup_warp_skips_if_exists() {
        let temp = TempDir::new().unwrap();
        let workspace = temp.path();
        let warp_md = workspace.join("WARP.md");

        // Create WARP.md with jumble section
        fs::write(&warp_md, format!("# WARP.md\n\n{}", JUMBLE_SECTION)).unwrap();

        // Should skip without --force
        setup_warp(workspace, false).unwrap();

        let content = fs::read_to_string(warp_md).unwrap();
        // Should only have one occurrence
        assert_eq!(content.matches(JUMBLE_SECTION_MARKER).count(), 1);
    }

    #[test]
    fn test_setup_warp_force_replaces() {
        let temp = TempDir::new().unwrap();
        let workspace = temp.path();
        let warp_md = workspace.join("WARP.md");

        // Create WARP.md with old jumble section
        let old_content = r#"# WARP.md

## Using Jumble for Project Context

This is old content that should be replaced.

## Other Section

Keep this.
"#;
        fs::write(&warp_md, old_content).unwrap();

        // Force update
        setup_warp(workspace, true).unwrap();

        let content = fs::read_to_string(warp_md).unwrap();
        assert!(content.contains("get_workspace_overview()"));
        assert!(!content.contains("This is old content"));
        assert!(content.contains("## Other Section"));
    }

    #[test]
    fn test_replace_jumble_section() {
        let content = r#"# WARP.md

## Using Jumble for Project Context

Old content here.

More old content.

## Another Section

Keep this section.
"#;

        let result = replace_jumble_section(content).unwrap();

        assert!(result.contains("get_workspace_overview()"));
        assert!(!result.contains("Old content here"));
        assert!(result.contains("## Another Section"));
    }
}
