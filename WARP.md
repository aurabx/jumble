# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Common commands

This is a single Rust crate built and run with Cargo. Canonical commands are also recorded in `.jumble/project.toml` under `[commands]`.

- Build (debug): `cargo build`
- Build (release, used in docs/examples): `cargo build --release`
- Run the MCP server (debug): `cargo run`
- Run all tests: `cargo test`
- Lint/format (as configured in `.jumble/project.toml`): `cargo clippy && cargo fmt --check`

### Running a single test

Tests are colocated in the same files as the code under `#[cfg(test)] mod tests`.

- Run tests in a single module file (e.g. `src/config.rs`): `cargo test config::tests`
- Run one specific test (e.g. `test_parse_minimal_project_config` in `config.rs`): `cargo test config::tests::test_parse_minimal_project_config`

### Running jumble from a specific root

The binary accepts a `--root` argument and also respects the `JUMBLE_ROOT` environment variable:

- Use CLI flag: `cargo run -- --root /path/to/workspace`
- Or via environment: `JUMBLE_ROOT=/path/to/workspace cargo run`

When packaged as a binary (e.g. via `cargo install --path .`), use the same flags/env when configuring MCP clients.

### Using jumble as an MCP server

Key snippets from `README.md` for wiring this binary into tools:

- Warp MCP config example (using a built binary on `PATH`):
  - Command: `jumble`
  - Args: `--root /path/to/your/workspace`
- Warp MCP config example (using a local build):
  - Command: `/path/to/repository/target/release/jumble`
  - Args: `--root /path/to/your/workspace`
- Claude Desktop MCP config: add a `"jumble"` entry pointing at the binary with `"args": ["--root", "/path/to/your/workspace"]` (see `README.md` for the exact JSON snippet).

## High-level architecture

### Purpose

`jumble` is an MCP server that scans a workspace for `.jumble` context files and exposes them to LLMs via JSON-RPC over stdio. Instead of streaming large documentation into the model, an agent calls structured tools (e.g. `get_workspace_overview`, `get_commands`) to pull only the relevant context.

### Top-level data model (`src/config.rs`)

`config.rs` defines the TOML-backed data structures that represent projects and workspaces:

- `ProjectConfig` is the main per-project structure, loaded from `.jumble/project.toml`.
  - `project: ProjectInfo` – basic metadata (name, description, optional language/version/repository).
  - `commands: HashMap<String, String>` – named commands like `build`, `test`, `lint`, etc.
  - `entry_points: HashMap<String, String>` – key files such as `src/main.rs`.
  - `dependencies: Dependencies` – `internal` vs `external` dependency lists.
  - `related_projects: RelatedProjects` – `upstream`/`downstream` relationships within a workspace.
  - `api: Option<ApiInfo>` – optional API surface description (OpenAPI spec, base URL, key endpoints).
  - `concepts: HashMap<String, Concept>` – architectural concepts mapped to file lists and summaries.
- `ProjectSkills` – map of skill topic → skill metadata, discovered from multiple sources:
  - `.jumble/skills/*.md` (flat project skills)
  - `.claude/skills/**/SKILL.md` (Claude structured skills)
  - `.codex/skills/**/SKILL.md` (Codex structured skills)
- `ProjectConventions` – per-project `conventions` and `gotchas`, loaded from `.jumble/conventions.toml`.
- `ProjectDocs` / `DocEntry` – index of documentation topics to paths and summaries from `.jumble/docs.toml`.
- `WorkspaceConfig` / `WorkspaceInfo` – workspace-level name/description plus shared `conventions` and `gotchas`, loaded from `.jumble/workspace.toml` at the workspace root.

These types are heavily unit-tested in `config.rs` to document the expected TOML shapes.

### JSON-RPC protocol layer (`src/protocol.rs`)

`protocol.rs` contains the minimal JSON-RPC types used by the MCP server:

- `JsonRpcRequest` – incoming request with `jsonrpc`, optional `id`, `method`, and `params` (as `serde_json::Value`).
- `JsonRpcResponse` – outgoing response with `jsonrpc`, optional `id`, optional `result`, and optional `error`.
- `JsonRpcError` – structured error payload with `code`, `message`, optional `data`.

Helpers `JsonRpcResponse::success` and `JsonRpcResponse::error` construct the two response variants. Unit tests exercise parsing/serialization behaviors (notifications without `id`, requests with/without `params`, success vs error responses).

### Server lifecycle and request handling (`src/server.rs`)

`server.rs` owns runtime state and provides the bridge between raw JSON-RPC requests and higher-level tools logic:

- `Server` struct holds:
  - `root: PathBuf` – workspace root to scan.
  - `workspace: Option<WorkspaceConfig>` – parsed `.jumble/workspace.toml` if present.
  - `projects: HashMap<String, ProjectData>` – all discovered projects keyed by project name.
- `Server::new(root: PathBuf)` sets up the server:
  - Loads workspace metadata via `load_workspace_static(root)` from `.jumble/workspace.toml`.
  - Calls `discover_projects()` to recursively walk the `root` with `walkdir` and find `.jumble/project.toml` files.
  - For each project directory, it:
    - Parses `.jumble/project.toml` into `ProjectConfig`.
    - Builds a `ProjectData` tuple containing:
      - The project root directory.
      - The parsed `ProjectConfig`.
      - `ProjectSkills` discovered from `.jumble/skills/*.md`, `.claude/skills/**/SKILL.md`, and `.codex/skills/**/SKILL.md`.
      - `ProjectConventions` loaded from `.jumble/conventions.toml` (or defaults).
      - `ProjectDocs` loaded from `.jumble/docs.toml` (or defaults).
- `handle_request` is the main dispatcher for JSON-RPC methods:
  - `"initialize"` → `handle_initialize` returns MCP capabilities and `serverInfo`.
  - `"initialized"` → acknowledges client initialization (no-op result).
  - `"tools/list"` → `handle_tools_list` delegates to `tools::tools_list()`.
  - `"tools/call"` → `handle_tools_call` validates `name` and `arguments`, then dispatches to a specific tool implementation in `tools.rs`.
  - Any other method yields a JSON-RPC "method not found" error.

Errors at this layer are expressed as `JsonRpcError` and packaged into `JsonRpcResponse` instances.

### Tools and workspace-level behaviors (`src/tools.rs`)

`tools.rs` holds both the **tool registry** (schema descriptions) and the implementations that operate over the loaded workspace/project data.

- `ProjectData` is a type alias shared with `server.rs`:
  - `(PathBuf, ProjectConfig, ProjectSkills, ProjectConventions, ProjectDocs)`.
- `tools_list()` returns a JSON schema describing all MCP tools exposed by this server (names, descriptions, and input JSON Schemas). This is what MCP clients call via `tools/list`.
- Each tool implementation takes `&HashMap<String, ProjectData>` (and optionally workspace data) plus `serde_json::Value` arguments and returns a `Result<String, String>` where the `String` is markdown meant to be shown to the user.

Key tools and what they do:

- `list_projects` – lists all projects with language, description, and filesystem path.
- `get_project_info` – returns either a high-level markdown summary (description, language, version, repository, entry points, concepts) or field-specific views (`commands`, `entry_points`, `dependencies`, `api`, `related_projects`).
- `get_commands` – returns formatted commands for a project, optionally filtered by `command_type` such as `build` or `test`.
- `get_architecture` – given a `project` and `concept`, locates the corresponding `Concept` in `.jumble/project.toml` and uses `format_concept` to show its description and file list. It supports exact, case-insensitive, and partial name matching.
- `get_related_files` – fuzzy search over concept names and summaries to find concepts related to a query, then aggregates their file lists.
- `list_skills` / `get_skill` – introspect skills from `.jumble/skills/*.md`, `.claude/skills/**/SKILL.md`, and `.codex/skills/**/SKILL.md`. Returns skill content and automatically lists companion files (scripts/, references/, docs/, assets/, examples/) for structured skills.
- `get_conventions` – surface per-project `conventions` and `gotchas` from `.jumble/conventions.toml`, with optional filtering by category.
- `get_docs` – expose `.jumble/docs.toml` either as an index of docs + summaries or as a detailed view of a single topic including its resolved path.
- `get_workspace_overview` – build a workspace-level summary: root path, all projects (name, language, description), and a simple textual dependency graph based on `related_projects`.
- `get_workspace_conventions` – like `get_conventions` but for workspace-level conventions/gotchas from `.jumble/workspace.toml`.

Tool outputs are intentionally formatted as markdown (headings, bold text, lists) to be directly consumable by LLM clients.

### Formatting helpers (`src/format.rs`)

`format.rs` provides small utilities for converting raw config structures into human-readable markdown strings:

- `format_commands` – renders the `[commands]` table as a bullet list, or a "No commands defined." message.
- `format_entry_points` – bullet list of tagged entry points.
- `format_dependencies` – splits dependencies into "Internal" vs "External" sections.
- `format_related_projects` – summarizes upstream/downstream workspace relationships.
- `format_api` – formats optional `ApiInfo` into OpenAPI path, base URL, and endpoints.
- `format_concept` – used by `get_architecture` to render a concept summary plus full file paths rooted at the project directory.

These helpers centralize the markdown output format so that tools remain focused on data selection and orchestration.

### Binary entrypoint and stdio loop (`src/main.rs`)

`main.rs` wires everything together and implements the stdio-based JSON-RPC loop expected by MCP clients:

- Defines CLI arguments using `clap::Parser` with a single optional `--root` (also bound to `JUMBLE_ROOT`). If neither is set, defaults to the current working directory.
- Constructs a `Server` with the resolved root directory.
- Reads stdin line-by-line using a buffered reader. Each non-empty line is treated as a complete JSON-RPC request.
- For each line:
  - Attempts to parse `JsonRpcRequest` with `serde_json::from_str`.
  - On parse failure, emits a `JsonRpcResponse` with `code = -32700` (parse error) and no `id`.
  - On success, passes the request to `server.handle_request` and serializes the resulting `JsonRpcResponse` back to stdout.
  - Flushes stdout after each response to keep the protocol well-behaved for streaming clients.

This design means **any** extra output to stdout (e.g. debug logs) can break the MCP protocol, so non-protocol logging must go to stderr.

## `.jumble` context files in this repo

This repository ships with its own `.jumble` metadata so that `jumble` can describe itself:

- `.jumble/project.toml` – describes this project (`jumble`) and defines canonical `commands`, entry points, external dependencies, and a few high-level `concepts.*`. Some concept entries still refer to older layouts (e.g. single-file `src/main.rs`), so treat them as a starting point rather than a complete architecture.
- `.jumble/conventions.toml` – captures project-specific conventions and gotchas. Some entries predate the current module layout but the following are still important:
  - `json_rpc_stdio` – the server speaks JSON-RPC over stdio; every request/response must be a single JSON line, and nothing else should be printed to stdout.
  - `markdown_output` – tools should return markdown-formatted strings (headings, bold, code blocks) for clarity in LLM clients.
  - `hashmap_ordering` – never rely on `HashMap` iteration order; sort keys before producing ordered output when order matters.
  - `path_resolution` – paths in `.jumble/*.toml` are relative to the project root containing `.jumble/` and are resolved to absolute paths at runtime.

When editing or extending `jumble`, keep `.jumble/project.toml` and `.jumble/conventions.toml` in sync with the Rust implementation so that tools like `get_workspace_overview`, `get_commands`, and `get_architecture` reflect reality.

## Authoring guide and schema

Two files at the repo root describe how `jumble` is meant to be used to model other projects:

- `AUTHORING.md` – detailed guide for creating `.jumble` context files (`project.toml`, `workspace.toml`, `conventions.toml`, `docs.toml`, and skills). It defines language-specific heuristics for discovering project names, descriptions, commands, entry points, dependencies, and concepts.
- `schema.json` – JSON Schema for validating `.jumble/*.toml` files (typically via tools like `taplo`).

When implementing new features or fixing bugs, refer to `AUTHORING.md` and the unit tests in `config.rs`/`tools.rs` to understand the intended shape and semantics of context files across different languages and ecosystems.



## Using Jumble for Project Context

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