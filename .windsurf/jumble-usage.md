# Using Jumble for Project Context

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
