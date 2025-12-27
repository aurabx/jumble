# Authoring Guide for `.jumble/project.toml`

This guide explains how to create a `.jumble/project.toml` file for a project. It's designed for both humans and AI assistants to follow.

## Quick Start

Create a `.jumble/` directory in your project root and add a `project.toml` file:

```toml
[project]
name = "my-project"
description = "One-line description of what this project does"
language = "rust"
```

That's the minimum. The sections below explain how to populate each field.

---

## [project] Section (Required)

### name
Use the canonical project name from:
1. **Rust**: `Cargo.toml` → `[package].name`
2. **Node**: `package.json` → `name`
3. **Python**: `pyproject.toml` → `[project].name` or `setup.py` name
4. **Go**: Module name from `go.mod`
5. **PHP**: `composer.json` → `name`
6. **Java**: `pom.xml` → `artifactId` or `build.gradle` project name
7. **Fallback**: Directory name

### description
Extract from:
1. Manifest file (Cargo.toml description, package.json description, etc.)
2. First paragraph of README.md
3. Write a concise one-liner if neither exists

Keep it under 100 characters. Focus on *what* the project does, not *how*.

### language
Detect from:
- Presence of `Cargo.toml` → `rust`
- Presence of `package.json` → `typescript` or `javascript`
- Presence of `pyproject.toml` / `setup.py` / `requirements.txt` → `python`
- Presence of `go.mod` → `go`
- Presence of `composer.json` → `php`
- Presence of `pom.xml` → `java`
- Presence of `build.gradle` / `build.gradle.kts` → `java` or `kotlin`

### version
Extract from manifest:
- Rust: `Cargo.toml` → `[package].version`
- Node: `package.json` → `version`
- Python: `pyproject.toml` → `[project].version`
- Go: Latest git tag or `v0.0.0`
- PHP: `composer.json` → `version`
- Java: `pom.xml` → `version`

### repository
Extract from:
1. Manifest (package.json repository, Cargo.toml repository)
2. `.git/config` → parse remote "origin" URL
3. Leave empty if not found

---

## [commands] Section

Provide commands for common development tasks. Always use the project's preferred tooling.

### Detection Heuristics by Language

#### Rust
```toml
[commands]
build = "cargo build --release"
test = "cargo test"
lint = "cargo clippy && cargo fmt --check"
run = "cargo run"
dev = "cargo watch -x run"  # if cargo-watch is used
```

#### Node/TypeScript (check package.json scripts)
```toml
[commands]
build = "pnpm build"        # or npm/yarn based on lockfile
test = "pnpm test"
lint = "pnpm lint"
dev = "pnpm dev"
```
**Detection**: Read `package.json` → `scripts` object. Use the actual script names.
**Package manager**: Prefer `pnpm` if `pnpm-lock.yaml` exists, `yarn` if `yarn.lock`, else `npm`.

#### Python
```toml
[commands]
build = "pip install -e ."
test = "pytest"
lint = "ruff check . && ruff format --check ."
run = "python -m mypackage"
```
**Detection**: Check for `pytest.ini`, `pyproject.toml` [tool.pytest], Makefile targets.

#### Go
```toml
[commands]
build = "go build ./..."
test = "go test ./..."
lint = "golangci-lint run"
run = "go run ."
```

#### PHP/Laravel
```toml
[commands]
build = "composer install"
test = "php artisan test"
lint = "./vendor/bin/pint --test"
run = "php artisan serve"
dev = "php artisan serve"
```
**Detection**: Check for `artisan` file (Laravel), `phpunit.xml`.

#### Java/Maven
```toml
[commands]
build = "mvn compile"
test = "mvn test"
lint = "mvn checkstyle:check"
run = "mvn exec:java"
```

#### Java/Gradle
```toml
[commands]
build = "./gradlew build"
test = "./gradlew test"
lint = "./gradlew check"
run = "./gradlew run"
```

### Makefile/Justfile Override
If a `Makefile` or `justfile` exists, prefer its targets:
```toml
[commands]
build = "make build"    # or "just build"
test = "make test"
lint = "make lint"
```

---

## [entry_points] Section

Identify key files that help understand the codebase structure.

### Common Patterns

| Language | Main Entry | Config | API |
|----------|-----------|--------|-----|
| Rust | `src/main.rs` or `src/lib.rs` | `src/config.rs` or `src/config/mod.rs` | `src/api/mod.rs` or `src/routes.rs` |
| Node | `src/index.ts` or `src/app.ts` | `src/config.ts` | `src/routes/index.ts` |
| Python | `src/__main__.py` or `app.py` | `config.py` | `app/routes.py` |
| Go | `main.go` or `cmd/*/main.go` | `config/config.go` | `internal/api/` |
| PHP/Laravel | `public/index.php` | `config/app.php` | `routes/api.php` |
| Java/Spring | `*Application.java` | `application.properties` | Controller classes |

### What to Include
- **main**: Primary entry point for execution
- **config**: Where configuration is loaded/defined
- **api**: API routes or handlers (for services)
- **models**: Data models or entities (if central to understanding)
- **core**: Core business logic module

Only include 3-5 most important entry points.

---

## [dependencies] Section

### internal
Other projects in the same workspace that this project depends on.

**Detection**:
- Rust: Check `Cargo.toml` for path dependencies
- Node: Check for workspace references in package.json
- Go: Check imports for local module paths

### external
Top 5-10 most important third-party dependencies.

**Selection criteria**:
- Framework dependencies (axum, express, django, laravel)
- Database clients (sqlx, prisma, sqlalchemy)
- Core functionality (serde, lodash, requests)

Don't list every dependency—focus on those that define the project's architecture.

---

## [related_projects] Section

### upstream
Projects this one consumes or depends on (from the same workspace).

### downstream
Projects that consume this one.

**Detection**: Cross-reference with other `.jumble/project.toml` files or workspace configuration.

---

## [api] Section (Optional)

Only include for services that expose an API.

### openapi
Path to OpenAPI/Swagger specification if it exists:
```toml
openapi = "docs/openapi.yaml"
```

### base_url
The API's base path:
```toml
base_url = "/api/v1"
```

### endpoints
List 5-10 most important endpoints (not exhaustive):
```toml
endpoints = [
    "GET /health",
    "POST /users",
    "GET /users/{id}",
    "POST /auth/login"
]
```

---

## [concepts.*] Section

Map architectural concepts to files. This is the most valuable section for AI context.

### How to Identify Concepts

1. **Directory structure**: `src/auth/` → `authentication` concept
2. **Module names**: `middleware.rs`, `validators.py` → concepts
3. **Domain areas**: `billing`, `notifications`, `search`

### What Makes a Good Concept

- **Focused**: One clear responsibility
- **Discoverable**: Files are grouped or clearly named
- **Important**: Central to understanding the architecture

### Examples

```toml
[concepts.authentication]
files = ["src/auth/mod.rs", "src/middleware/jwt.rs"]
summary = "JWT-based authentication with middleware validation"

[concepts.database]
files = ["src/db/mod.rs", "src/db/migrations/", "src/models/"]
summary = "PostgreSQL with SQLx, migrations in SQL files"

[concepts.api_routing]
files = ["src/routes/mod.rs", "src/handlers/"]
summary = "Axum router with handler functions per resource"
```

### Guidelines

- Include 3-5 concepts (more for complex projects)
- Keep summaries to one sentence
- Use relative paths from project root
- Directories can be listed (e.g., `src/models/`)

---

## Complete Example

```toml
[project]
name = "harmony-proxy"
description = "Secure reverse proxy for healthcare data mesh"
language = "rust"
version = "0.2.1"
repository = "https://github.com/org/harmony-proxy"

[commands]
build = "cargo build --release"
test = "cargo test"
lint = "cargo clippy && cargo fmt --check"
run = "cargo run"
dev = "cargo watch -x run"

[entry_points]
main = "src/main.rs"
config = "src/config/mod.rs"
api = "src/routes/mod.rs"

[dependencies]
internal = ["harmony-dsl", "jolt-rs"]
external = ["tokio", "axum", "sqlx", "serde", "tracing"]

[related_projects]
upstream = ["harmony-dsl"]
downstream = ["harmony-examples"]

[api]
openapi = "docs/openapi.yaml"
base_url = "/api/v1"
endpoints = [
    "GET /health",
    "POST /transform",
    "GET /config",
    "POST /validate"
]

[concepts.authentication]
files = ["src/auth/mod.rs", "src/middleware/jwt.rs"]
summary = "JWT validation via middleware, tokens from external IdP"

[concepts.transformation]
files = ["src/transform/mod.rs", "src/transform/pipeline.rs"]
summary = "Data transformation pipeline using Jolt specifications"

[concepts.routing]
files = ["src/routes/mod.rs", "src/routes/proxy.rs"]
summary = "Axum router with dynamic backend routing based on config"

[concepts.configuration]
files = ["src/config/mod.rs", "src/config/loader.rs"]
summary = "TOML-based config with hot reload support"
```

---

## Validation

Use the JSON schema at `schema.json` to validate your TOML:

```bash
# With taplo (TOML toolkit)
taplo check .jumble/project.toml --schema ../path/to/jumble/schema.json
```
