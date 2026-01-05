# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.0] - 2026-01-05

### Added
- **Global Jumble config**: Optional per-user config file at `~/.jumble/jumble.toml` that can hold shared defaults and settings. If the file does not exist, Jumble creates it on first run with a minimal `[jumble]` section reserved for future options.
- **Global skills support**: Flat skills discovered from `~/.jumble/skills/*.md` in addition to project-local `.jumble/skills/*.md`. Global skills are available in every workspace.

### Changed
- Server startup now attempts to load the global config and global skills; failures to create, read, or parse these files are logged but do not prevent the server from starting.
- Skills discovery now merges project-local and global skills, with project-local skills taking precedence when a global skill has the same name.

### Documentation
- Updated README.md and AUTHORING.md to document global config (`~/.jumble/jumble.toml`) and personal/global skills in `~/.jumble/skills/*.md`.

## [0.4.0] - 2026-01-02

### Added
- **Codex skills support**: Jumble now autodiscovers skills from `.codex/skills/**/SKILL.md` (both project-local and `$HOME/.codex/skills`), in addition to existing `.claude/skills` support.
- **Companion files for structured skills**: Skills in SKILL.md format can now include companion resources like `scripts/`, `references/`, `docs/`, `assets/`, `examples/`, and `templates/` subdirectories. When retrieving a skill via `get_skill`, companion files are automatically discovered and listed.
- Comprehensive test coverage for companion file discovery with 4 new tests.

### Changed
- **Renamed "prompts" to "skills" throughout the codebase**:
  - MCP tools: `list_prompts` → `list_skills`, `get_prompt` → `get_skill`
  - Types: `PromptFrontmatter` → `SkillFrontmatter`, `PromptInfo` → `SkillInfo`, `ProjectPrompts` → `ProjectSkills`
  - Directory: `.jumble/prompts/` → `.jumble/skills/`
  - All documentation updated to reflect the new terminology
- Enhanced `SkillInfo` to track the skill directory for accessing companion files.
- Renamed internal function `discover_claude_skills_in_dir` → `discover_structured_skills_in_dir` to reflect support for both Claude and Codex formats.

### Documentation
- Updated README.md, AUTHORING.md, and WARP.md to document `.codex` support and companion file features.
- Clarified that Jumble is compatible with both Claude and Codex skill formats without requiring users to rewrite existing skills.

## [0.3.0] - 2026-01-02

### Added
- New `reload_workspace` MCP tool to reload `.jumble` workspace and project metadata from disk without restarting the server.
- New `get_jumble_authoring_prompt` MCP tool that returns a canonical prompt for generating `.jumble` context files in any project or workspace.

### Changed
- Workspace and project discovery is now performed once at startup and cached in memory, rather than rescanning the filesystem on every tools call.
- Updated README to describe the cached behavior and the `reload_workspace` tool.
- Simplified the GitHub Actions release workflow to build multi-platform binaries and publish them as GitHub Releases, removing the Docker image build step.

## [0.2.0]

- Initial public release of `jumble` as an MCP server for workspace-aware project context.
