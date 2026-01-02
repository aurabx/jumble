# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2026-01-02

### Added
- New `reload_workspace` MCP tool to reload `.jumble` workspace and project metadata from disk without restarting the server.

### Changed
- Workspace and project discovery is now performed once at startup and cached in memory, rather than rescanning the filesystem on every tools call.
- Updated README to describe the cached behavior and the `reload_workspace` tool.
- Simplified the GitHub Actions release workflow to build multi-platform binaries and publish them as GitHub Releases, removing the Docker image build step.

## [0.2.0]

- Initial public release of `jumble` as an MCP server for workspace-aware project context.
