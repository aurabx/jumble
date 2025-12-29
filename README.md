# jumble

An MCP server that provides queryable, on-demand project context to LLMs.

---

<div align="center">

### 🛠️ This project was built by [Aurabox](https://github.com/aurabx)

*Some of our other projects...*

[![Harmony](https://img.shields.io/badge/Harmony-API_Gateway-purple?style=flat-square)](https://github.com/aurabox/harmony-proxy)
[![Runbeam](https://img.shields.io/badge/Runbeam-Data_Mesh_Platform-purple?style=flat-square)](https://runbeam.io)
[![JMIX](https://img.shields.io/badge/Aurabox-Medical_Imaging_Exchange-blue?style=flat-square)](https://aurabox.cloud)

</div>

---

## The Problem

Large documentation files and WARP.md overload LLM context windows. Even well-structured docs require reading everything upfront, wasting tokens on irrelevant information.

## The Solution

Jumble flips the model: instead of loading documentation, an LLM *queries* for exactly what it needs.

```
LLM: "What projects are in this workspace?"
     → calls get_workspace_overview()
     → receives: workspace info, all projects, dependency graph

LLM: "What's the test command for my-app"
     → calls get_commands("my-app", "test")
     → receives: "cargo test"
     
LLM: "What files handle authentication?"
     → calls get_architecture("my-app", "authentication")
     → receives: files + one-sentence summary
     
LLM: "What conventions should I follow?"
     → calls get_workspace_conventions()
     → receives: workspace-wide coding standards
```

## Installation

### From source

```bash
cargo install --path .
```

### From crates.io

```bash
cargo install jumble
```

## Configuration

Jumble discovers projects by scanning for `.jumble/project.toml` files. It also looks for a `.jumble/workspace.toml` at the root for workspace-level configuration.

Set the root directory via:

1. `JUMBLE_ROOT` environment variable
2. `--root` CLI argument
3. Current working directory (default)

## Usage with Warp

Add to your Warp MCP configuration:

```json
{
  "jumble": {
    "command": "jumble",
    "args": ["--root", "/path/to/your/workspace"]
  }
}
```
or, if you are building from source...

```json
{
  "jumble": {
    "args": [
      "--root",
      "/path/to/your/workspace"
    ],
    "command": "/<path/to/repository>/target/release/jumble"
  }
}
```

## Usage with Claude Desktop

Add to `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "jumble": {
      "command": "/path/to/jumble",
      "args": ["--root", "/path/to/your/workspace"]
    }
  }
}
```

## Creating Context Files

Context files are designed to be created by the same AI agents that read them. See See [AUTHORING.md](AUTHORING.md) for the complete guide.

Sample prompt:
```
Create jumble context for this project.

Read the AUTHORING.md guide at /path/to/jumble/AUTHORING.md, then examine this project's structure to create:

1. .jumble/project.toml (required) - Extract project info from manifest files, identify key commands, map architectural concepts to files
2. .jumble/conventions.toml - Capture patterns to follow and gotchas to avoid (look at existing code patterns, comments, and any constitution.md or similar files)
3. .jumble/docs.toml - Index the docs/ directory if it exists, with one-line summaries
```

### Project Context

Create a `.jumble/project.toml` in each project:

```toml
[project]
name = "my-project"
description = "One-line description"
language = "rust"

[commands]
build = "cargo build --release"
test = "cargo test"
lint = "cargo clippy"

[entry_points]
main = "src/main.rs"

[concepts.authentication]
files = ["src/auth/mod.rs"]
summary = "JWT-based auth via middleware"

[related_projects]
upstream = ["shared-lib"]    # projects this depends on
downstream = ["examples"]    # projects that depend on this
```

### Workspace Context

Create a `.jumble/workspace.toml` at the workspace root:

```toml
[workspace]
name = "My Workspace"
description = "Monorepo for my projects"

[conventions]
error_handling = "Use anyhow for apps, thiserror for libraries"
testing = "Unit tests in same file, integration tests in tests/"

[gotchas]
feature_flags = "Features enabled by one project affect all dependents"
```

### Optional Files

- `.jumble/conventions.toml` - Project-specific conventions and gotchas
- `.jumble/docs.toml` - Documentation index with summaries
- `.jumble/prompts/*.md` - Task-specific prompts for common operations

See [AUTHORING.md](AUTHORING.md) for the complete guide.

## Available Tools

### Workspace Tools

#### get_workspace_overview
Returns workspace info, all projects with descriptions, and dependency graph. **Call this first** to understand the workspace structure.

```
get_workspace_overview()
```

#### get_workspace_conventions
Returns workspace-level conventions and gotchas that apply across all projects.

```
get_workspace_conventions()
get_workspace_conventions(category: "gotchas")
```

### Project Tools

#### list_projects
Lists all discovered projects with their descriptions.

#### get_project_info
Returns metadata about a project (description, language, version, entry points).

```
get_project_info(project: "my-project")
get_project_info(project: "my-project", field: "dependencies")
```

#### get_commands
Returns executable commands for a project.

```
get_commands(project: "my-project")
get_commands(project: "my-project", command_type: "test")
```

#### get_architecture
Returns files and summary for a specific architectural concept.

```
get_architecture(project: "my-project", concept: "authentication")
```

#### get_related_files
Searches concepts and returns matching files.

```
get_related_files(project: "my-project", query: "database")
```

#### get_conventions
Returns project-specific coding conventions and gotchas.

```
get_conventions(project: "my-project")
get_conventions(project: "my-project", category: "gotchas")
```

#### get_docs
Returns documentation index with summaries, or path to a specific doc.

```
get_docs(project: "my-project")
get_docs(project: "my-project", topic: "configuration")
```

#### list_prompts / get_prompt
Lists or retrieves task-specific prompts for common operations.

```
list_prompts(project: "my-project")
get_prompt(project: "my-project", topic: "add-endpoint")
```

## AI-Assisted Authoring

Jumble is designed so that an AI can generate context files for any project:

1. **schema.json** - Machine-readable schema for validation
2. **AUTHORING.md** - Heuristics for how to populate each field

When asked to "create jumble context for project X", an AI should:
1. Read AUTHORING.md to understand the heuristics
2. Examine the project's manifest files, directory structure, and README
3. Generate `.jumble/project.toml` (required)
4. Optionally generate `conventions.toml`, `docs.toml`, and prompts

## Schema Validation

Validate your TOML files with the included JSON schema:

```bash
# With taplo
taplo check .jumble/project.toml --schema /path/to/jumble/schema.json
```

## License

MIT

---

<div align="center">

### 🛠️ This project was built by [Aurabox](https://github.com/aurabx)

*Some of our other projects...*

[![Harmony](https://img.shields.io/badge/Harmony-API_Gateway-purple?style=flat-square)](https://github.com/aurabox/harmony-proxy)
[![Runbeam](https://img.shields.io/badge/Runbeam-Data_Mesh_Platform-purple?style=flat-square)](https://runbeam.io)
[![JMIX](https://img.shields.io/badge/Aurabox-Medical_Imaging_Exchange-blue?style=flat-square)](https://aurabox.cloud)

</div>