//! Formatting helpers for output strings.

use crate::config::{ApiInfo, Concept, Dependencies, RelatedProjects};
use std::collections::HashMap;
use std::path::Path;

pub fn format_commands(commands: &HashMap<String, String>) -> String {
    if commands.is_empty() {
        return "No commands defined.".to_string();
    }
    let mut output = String::new();
    for (name, cmd) in commands {
        output.push_str(&format!("- **{}**: `{}`\n", name, cmd));
    }
    output
}

pub fn format_entry_points(entry_points: &HashMap<String, String>) -> String {
    if entry_points.is_empty() {
        return "No entry points defined.".to_string();
    }
    let mut output = String::new();
    for (name, path) in entry_points {
        output.push_str(&format!("- **{}**: {}\n", name, path));
    }
    output
}

pub fn format_dependencies(deps: &Dependencies) -> String {
    let mut output = String::new();
    if !deps.internal.is_empty() {
        output.push_str("**Internal dependencies:**\n");
        for dep in &deps.internal {
            output.push_str(&format!("- {}\n", dep));
        }
    }
    if !deps.external.is_empty() {
        output.push_str("**External dependencies:**\n");
        for dep in &deps.external {
            output.push_str(&format!("- {}\n", dep));
        }
    }
    if output.is_empty() {
        "No dependencies defined.".to_string()
    } else {
        output
    }
}

pub fn format_related_projects(related: &RelatedProjects) -> String {
    let mut output = String::new();
    if !related.upstream.is_empty() {
        output.push_str("**Upstream (this project depends on):**\n");
        for proj in &related.upstream {
            output.push_str(&format!("- {}\n", proj));
        }
    }
    if !related.downstream.is_empty() {
        output.push_str("**Downstream (depends on this project):**\n");
        for proj in &related.downstream {
            output.push_str(&format!("- {}\n", proj));
        }
    }
    if output.is_empty() {
        "No related projects defined.".to_string()
    } else {
        output
    }
}

pub fn format_api(api: &Option<ApiInfo>) -> String {
    match api {
        Some(api_info) => {
            let mut output = String::new();
            if let Some(openapi) = &api_info.openapi {
                output.push_str(&format!("**OpenAPI spec:** {}\n", openapi));
            }
            if let Some(base_url) = &api_info.base_url {
                output.push_str(&format!("**Base URL:** {}\n", base_url));
            }
            if !api_info.endpoints.is_empty() {
                output.push_str("**Endpoints:**\n");
                for endpoint in &api_info.endpoints {
                    output.push_str(&format!("- {}\n", endpoint));
                }
            }
            if output.is_empty() {
                "API section defined but empty.".to_string()
            } else {
                output
            }
        }
        None => "No API information defined.".to_string(),
    }
}

pub fn format_concept(project_path: &Path, name: &str, concept: &Concept) -> String {
    let mut output = format!("## {}\n\n{}\n\n**Files:**\n", name, concept.summary);
    for file in &concept.files {
        output.push_str(&format!("- {}/{}\n", project_path.display(), file));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_commands_empty() {
        let commands = HashMap::new();
        assert_eq!(format_commands(&commands), "No commands defined.");
    }

    #[test]
    fn test_format_commands() {
        let mut commands = HashMap::new();
        commands.insert("build".to_string(), "cargo build".to_string());
        
        let result = format_commands(&commands);
        assert!(result.contains("**build**"));
        assert!(result.contains("`cargo build`"));
    }

    #[test]
    fn test_format_entry_points_empty() {
        let entry_points = HashMap::new();
        assert_eq!(format_entry_points(&entry_points), "No entry points defined.");
    }

    #[test]
    fn test_format_entry_points() {
        let mut entry_points = HashMap::new();
        entry_points.insert("main".to_string(), "src/main.rs".to_string());
        
        let result = format_entry_points(&entry_points);
        assert!(result.contains("**main**"));
        assert!(result.contains("src/main.rs"));
    }

    #[test]
    fn test_format_dependencies_empty() {
        let deps = Dependencies::default();
        assert_eq!(format_dependencies(&deps), "No dependencies defined.");
    }

    #[test]
    fn test_format_dependencies_internal_only() {
        let deps = Dependencies {
            internal: vec!["shared-lib".to_string()],
            external: vec![],
        };
        
        let result = format_dependencies(&deps);
        assert!(result.contains("Internal dependencies"));
        assert!(result.contains("shared-lib"));
        assert!(!result.contains("External dependencies"));
    }

    #[test]
    fn test_format_dependencies_both() {
        let deps = Dependencies {
            internal: vec!["core".to_string()],
            external: vec!["serde".to_string(), "tokio".to_string()],
        };
        
        let result = format_dependencies(&deps);
        assert!(result.contains("Internal dependencies"));
        assert!(result.contains("External dependencies"));
        assert!(result.contains("serde"));
    }

    #[test]
    fn test_format_related_projects_empty() {
        let related = RelatedProjects::default();
        assert_eq!(format_related_projects(&related), "No related projects defined.");
    }

    #[test]
    fn test_format_related_projects() {
        let related = RelatedProjects {
            upstream: vec!["core-lib".to_string()],
            downstream: vec!["frontend".to_string()],
        };
        
        let result = format_related_projects(&related);
        assert!(result.contains("Upstream"));
        assert!(result.contains("core-lib"));
        assert!(result.contains("Downstream"));
        assert!(result.contains("frontend"));
    }

    #[test]
    fn test_format_api_none() {
        assert_eq!(format_api(&None), "No API information defined.");
    }

    #[test]
    fn test_format_api_with_data() {
        let api = Some(ApiInfo {
            openapi: Some("api.yaml".to_string()),
            base_url: Some("/api/v1".to_string()),
            endpoints: vec!["GET /users".to_string()],
        });
        
        let result = format_api(&api);
        assert!(result.contains("OpenAPI spec"));
        assert!(result.contains("api.yaml"));
        assert!(result.contains("Base URL"));
        assert!(result.contains("/api/v1"));
        assert!(result.contains("Endpoints"));
    }

    #[test]
    fn test_format_api_empty() {
        let api = Some(ApiInfo {
            openapi: None,
            base_url: None,
            endpoints: vec![],
        });
        
        assert_eq!(format_api(&api), "API section defined but empty.");
    }

    #[test]
    fn test_format_concept() {
        let concept = Concept {
            files: vec!["src/auth.rs".to_string(), "src/jwt.rs".to_string()],
            summary: "Authentication module".to_string(),
        };
        let path = Path::new("/project");
        
        let result = format_concept(path, "authentication", &concept);
        assert!(result.contains("## authentication"));
        assert!(result.contains("Authentication module"));
        assert!(result.contains("/project/src/auth.rs"));
        assert!(result.contains("/project/src/jwt.rs"));
    }
}
