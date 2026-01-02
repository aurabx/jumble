# jumble

An MCP server that provides queryable, on-demand project context to LLMs.

## The Problem

Large documentation files overload LLM context windows. Even well-structured docs require reading everything upfront, wasting tokens on irrelevant information.

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

### Prebuilt binaries

Prebuilt binaries for common platforms (Linux, macOS, Windows) are available on the GitHub Releases page:

- https://github.com/velvet-tiger/jumble/releases/

Download the archive for your platform, extract it, and point your MCP client at the extracted `jumble` binary.

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

Projects and workspace metadata are loaded once when the server starts and cached in memory. If you change any `.jumble/*` files, either restart the `jumble` process or call the `reload_workspace` tool (see below) to pick up changes without restarting.

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

Jumble supports loading Claude Skills.md files.

## Usage with Windsurf

Windsurf's Cascade MCP config uses the same shape as Claude Desktop's config. Create or edit `~/.codeium/windsurf/mcp_config.json`:

```json
{
  "mcpServers": {
    "jumble": {
      "command": "/absolute/path/to/jumble",
      "args": ["--root", "/path/to/your/workspace"]
    }
  }
}
```

Restart Windsurf so Cascade reloads MCP servers, then verify that `jumble` appears in the tools list.

## Usage with Cursor

Cursor reads MCP configuration from `mcp.json` in either your project `.cursor` directory or your home directory.

Global configuration (available in all projects), in `~/.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "jumble": {
      "command": "/absolute/path/to/jumble",
      "args": ["--root", "/path/to/your/workspace"]
    }
  }
}
```

Alternatively, add the same `mcpServers` block to `.cursor/mcp.json` in a single project to scope `jumble` to that project only.

## Usage with Codex

Codex stores MCP configuration in `~/.codex/config.toml`.

Using the Codex CLI:

```bash
codex mcp add jumble -- /absolute/path/to/jumble --root /path/to/your/workspace
```

Or edit `~/.codex/config.toml` directly:

```toml
[mcp_servers.jumble]
command = "/absolute/path/to/jumble"
args = ["--root", "/path/to/your/workspace"]
```

Restart the Codex IDE extension or TUI and confirm that `jumble` is listed as an MCP server.

## Creating Context Files

Context files are designed to be created by the same AI agents that read them. See [AUTHORING.md](AUTHORING.md) for the complete guide.

Sample prompt:
```
Create jumble context for this project.

Read the AUTHORING.md guide at /path/to/jumble/AUTHORING.md, then examine this project's structure to create:

1. .jumble/project.toml (required) - Extract project info from manifest files, identify key commands, map architectural concepts to files
2. .jumble/conventions.toml - Capture patterns to follow and gotchas to avoid (look at existing code patterns, comments, and any constitution.md or similar files)
3. .jumble/docs.toml - Index the docs/ directory if it exists, with one-line summaries
```

## Core Jumble concepts

These fields appear in `.jumble/project.toml` / `.jumble/workspace.toml` and are what the MCP tools expose back to the AI.

- **Commands** (`[commands]`)
  - A small map of named, copy-pastable CLI commands (e.g. `build`, `test`, `lint`, `run`, `dev`).
  - Used by `get_commands(project, command_type)` and `get_project_info(..., field: "commands")` so the AI can tell you *exactly* how to build, test, or run a project without guessing.

- **Entry points** (`[entry_points]`)
  - Labels to the most important source files for a project (e.g. `main = "src/main.rs"`, `api = "src/api/mod.rs"`).
  - Surface in `get_project_info(project)` so an AI can jump straight to the right file when exploring a new codebase.

- **Concepts / architecture** (`[concepts.*]`)
  - Each concept describes an architectural area with a short summary and a list of files, for example:
    - `concepts.authentication.files = ["src/auth/mod.rs"]`
    - `concepts.authentication.summary = "JWT-based auth via middleware"`
  - `get_architecture(project, concept)` returns the full description and file list for one concept.
  - `get_related_files(project, query)` searches across all concepts by name/summary to find related files (e.g. "database", "routing").

- **Documentation index** (`.jumble/docs.toml`)
  - A lightweight index of human-written docs (README, design docs, ADRs, etc.) with one-line summaries.
  - `get_docs(project)` lists all topics and summaries so the AI can pick the right document before reading it.
  - `get_docs(project, topic)` returns the resolved filesystem path for a single doc.

- **Coding conventions and gotchas** (`.jumble/conventions.toml` and workspace `[conventions]` / `[gotchas]`)
  - **Project-level** `conventions.toml` captures patterns to follow and sharp edges to avoid for a single project.
  - **Workspace-level** `[conventions]` / `[gotchas]` in `.jumble/workspace.toml` describe cross-project standards and pitfalls.
  - `get_conventions(project, ...)` returns project-specific conventions/gotchas.
  - `get_workspace_conventions(...)` returns workspace-wide standards or gotchas.

- **Related projects** (`[related_projects]`)
  - Describes how projects in the same workspace depend on each other:
    - `upstream = ["shared-lib"]` → projects this one depends on.
    - `downstream = ["examples"]` → projects that depend on this one.
  - `get_workspace_overview()` uses this to build a simple textual dependency graph so the AI can see how projects fit together.

- **Workspaces** (`.jumble/workspace.toml`)
  - Describes the overall workspace/monorepo: name, description, and shared conventions/gotchas.
  - `get_workspace_overview()` returns workspace metadata plus the list of all projects.
  - `get_workspace_conventions()` returns workspace-level conventions and gotchas that apply across multiple projects.

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

#### reload_workspace
Reloads workspace and project metadata from disk. Use this after editing `.jumble` files if you want to avoid restarting the MCP server.

```
reload_workspace()
```

#### get_jumble_authoring_prompt
Returns a canonical prompt you can feed to an AI assistant to generate `.jumble` context files (project, workspace, conventions, docs) for any project.

```
get_jumble_authoring_prompt()
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