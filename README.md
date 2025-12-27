# jumble

An MCP server that provides queryable, on-demand project context to LLMs.

## The Problem

Large documentation files and WARP.md overload LLM context windows. Even well-structured docs require reading everything upfront, wasting tokens on irrelevant information.

## The Solution

Jumble flips the model: instead of loading documentation, an LLM *queries* for exactly what it needs.

```
LLM: "What's the test command for harmony-proxy?"
     → calls get_commands("harmony-proxy", "test")
     → receives: "cargo test"
     
LLM: "What files handle authentication?"
     → calls get_architecture("harmony-proxy", "authentication")
     → receives: files + one-sentence summary
```

5 tokens instead of 500.

## Installation

### From source

```bash
cargo install --path .
```

### From crates.io (coming soon)

```bash
cargo install jumble
```

## Configuration

Jumble discovers projects by scanning for `.jumble/project.toml` files.

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

## Creating Project Context

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
```

See [AUTHORING.md](AUTHORING.md) for the complete guide on populating these files.

## Available Tools

### list_projects
Lists all discovered projects with their descriptions.

### get_project_info
Returns metadata about a project (description, language, version, entry points).

```
get_project_info(project: "my-project")
get_project_info(project: "my-project", field: "dependencies")
```

### get_commands
Returns executable commands for a project.

```
get_commands(project: "my-project")
get_commands(project: "my-project", command_type: "test")
```

### get_architecture
Returns files and summary for a specific architectural concept.

```
get_architecture(project: "my-project", concept: "authentication")
```

### get_related_files
Searches concepts and returns matching files.

```
get_related_files(project: "my-project", query: "database")
```

## AI-Assisted Authoring

Jumble is designed so that an AI can generate `.jumble/project.toml` files for any project:

1. **schema.json** - Machine-readable schema for validation
2. **AUTHORING.md** - Heuristics for how to populate each field

When asked to "create jumble context for project X", an AI should:
1. Read AUTHORING.md to understand the heuristics
2. Examine the project's manifest files, directory structure, and README
3. Generate a valid `.jumble/project.toml`

## Schema Validation

Validate your TOML files with the included JSON schema:

```bash
# With taplo
taplo check .jumble/project.toml --schema /path/to/jumble/schema.json
```

## License

MIT
