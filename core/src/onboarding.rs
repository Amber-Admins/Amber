use serde::{Deserialize, Serialize};

const ALLOWED_NODE_TYPES: [&str; 8] = [
    "concept",
    "fact",
    "project",
    "preference",
    "event",
    "instruction",
    "identity",
    "summary",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProposedNode {
    pub title: String,
    pub summary: String,
    pub detail: Option<String>,
    pub category: Option<String>,
    pub target_vault_key: Option<String>,
    pub tags: Option<Vec<String>>,
    pub node_type: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProposalEnvelope {
    proposals: Vec<RawProposedNode>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawProposedNode {
    title: String,
    summary: String,
    #[serde(default)]
    detail: Option<String>,
    #[serde(default)]
    category: Option<String>,
    #[serde(default)]
    target_vault_key: Option<String>,
    #[serde(default)]
    tags: Option<Vec<String>>,
    #[serde(default)]
    node_type: Option<String>,
}

fn normalize_non_empty(value: Option<String>) -> Option<String> {
    value
        .map(|raw| raw.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn normalize_required(value: String, field_name: &str, index: usize) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(format!(
            "Proposal {} has empty required field '{}'",
            index + 1,
            field_name
        ));
    }
    Ok(trimmed.to_string())
}

fn validate_node_type(value: Option<String>, index: usize) -> Result<Option<String>, String> {
    let Some(raw_value) = value else {
        return Ok(None);
    };
    let normalized = raw_value.trim().to_lowercase();
    if normalized.is_empty() {
        return Ok(None);
    }
    if ALLOWED_NODE_TYPES.contains(&normalized.as_str()) {
        Ok(Some(normalized))
    } else {
        Err(format!(
            "Proposal {} has unsupported node_type '{}'",
            index + 1,
            raw_value
        ))
    }
}

fn normalize_tags(tags: Option<Vec<String>>, index: usize) -> Result<Option<Vec<String>>, String> {
    let Some(values) = tags else {
        return Ok(None);
    };

    let mut normalized = Vec::new();
    for tag in values {
        let trimmed = tag.trim();
        if trimmed.is_empty() {
            return Err(format!("Proposal {} includes an empty tag", index + 1));
        }
        normalized.push(trimmed.to_string());
    }

    if normalized.is_empty() {
        Ok(None)
    } else {
        Ok(Some(normalized))
    }
}

/// Parse strict onboarding proposal JSON for both interview extraction and paste import.
/// The accepted payload shape is:
/// `{ "proposals": [ { "title": "...", "summary": "...", ... } ] }`
pub fn parse_proposals_json(raw_json: &str) -> Result<Vec<ProposedNode>, String> {
    let envelope: ProposalEnvelope = serde_json::from_str(raw_json)
        .map_err(|err| format!("Invalid onboarding proposals JSON: {err}"))?;

    if envelope.proposals.is_empty() {
        return Err("Onboarding proposals payload is empty".to_string());
    }

    envelope
        .proposals
        .into_iter()
        .enumerate()
        .map(|(index, raw)| {
            let title = normalize_required(raw.title, "title", index)?;
            let summary = normalize_required(raw.summary, "summary", index)?;
            let detail = normalize_non_empty(raw.detail);
            let category = normalize_non_empty(raw.category).map(|v| v.to_lowercase());
            let target_vault_key =
                normalize_non_empty(raw.target_vault_key).map(|v| v.to_lowercase());
            if category.is_none() && target_vault_key.is_none() {
                return Err(format!(
                    "Proposal {} must include 'category' or 'target_vault_key'",
                    index + 1
                ));
            }

            let tags = normalize_tags(raw.tags, index)?;
            let node_type = validate_node_type(raw.node_type, index)?;

            Ok(ProposedNode {
                title,
                summary,
                detail,
                category,
                target_vault_key,
                tags,
                node_type,
            })
        })
        .collect()
}

/// Stable category keys used by onboarding prompts and import parsers.
/// Keep these aligned with `db/migrations/0003_onboarding_default_vaults.sql`.
pub fn vault_id_for_category_key(category_key: &str) -> Option<&'static str> {
    match category_key.trim().to_lowercase().as_str() {
        "demographics" => Some("vault_root_graph"),
        "interests" | "personal" => Some("vault_personal"),
        "work" => Some("vault_work"),
        "learning" => Some("vault_learning"),
        "health" => Some("vault_health"),
        "finance" => Some("vault_finance"),
        "credentials" => Some("vault_credentials"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_proposals_json, vault_id_for_category_key};

    #[test]
    fn parse_valid_proposals_golden() {
        let payload = r#"{
  "proposals": [
    {
      "title": "Primary work focus",
      "summary": "Leading customer onboarding revamp project",
      "detail": "Cross-functional rollout tracked weekly",
      "category": "work",
      "tags": ["project", "priority"],
      "node_type": "project"
    },
    {
      "title": "Prefers short daily workouts",
      "summary": "20-minute routines are easiest to sustain",
      "target_vault_key": "health"
    }
  ]
}"#;

        let parsed = match parse_proposals_json(payload) {
            Ok(value) => value,
            Err(err) => panic!("expected valid proposals payload: {err}"),
        };

        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].title, "Primary work focus");
        assert_eq!(parsed[0].category.as_deref(), Some("work"));
        assert_eq!(parsed[0].node_type.as_deref(), Some("project"));
        assert_eq!(parsed[1].target_vault_key.as_deref(), Some("health"));
    }

    #[test]
    fn parse_rejects_trailing_junk() {
        let payload =
            r#"{"proposals":[{"title":"A","summary":"B","category":"personal"}]} trailing"#;
        let err = match parse_proposals_json(payload) {
            Ok(_) => panic!("expected trailing junk payload to fail"),
            Err(value) => value,
        };
        assert!(err.contains("Invalid onboarding proposals JSON"));
    }

    #[test]
    fn parse_rejects_missing_required_field() {
        let payload = r#"{"proposals":[{"title":"A","category":"personal"}]}"#;
        let err = match parse_proposals_json(payload) {
            Ok(_) => panic!("expected missing required field payload to fail"),
            Err(value) => value,
        };
        assert!(err.contains("Invalid onboarding proposals JSON"));
    }

    #[test]
    fn parse_rejects_without_category_or_target() {
        let payload = r#"{"proposals":[{"title":"A","summary":"B"}]}"#;
        let err = match parse_proposals_json(payload) {
            Ok(_) => panic!("expected category/target validation to fail"),
            Err(value) => value,
        };
        assert!(err.contains("must include 'category' or 'target_vault_key'"));
    }

    #[test]
    fn vault_map_resolves_expected_keys() {
        assert_eq!(
            vault_id_for_category_key("personal"),
            Some("vault_personal")
        );
        assert_eq!(
            vault_id_for_category_key("Demographics"),
            Some("vault_root_graph")
        );
        assert_eq!(vault_id_for_category_key("unknown"), None);
    }
}
