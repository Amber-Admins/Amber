use crate::ipc_types::ChangesetCommitInput;
use crate::priority;
use crate::redacted;
use rusqlite::{params, Connection, Transaction};
use std::collections::HashSet;
use std::path::Path;

fn resolve_effective_privacy(
    tx: &rusqlite::Transaction,
    vault_id: &str,
    privacy_override: Option<&str>,
) -> Result<String, String> {
    if let Some(tier) = privacy_override {
        return Ok(tier.to_string());
    }

    let vault_privacy: String = tx
        .query_row(
            "SELECT privacy_tier FROM vaults WHERE id = ?1 LIMIT 1;",
            [vault_id],
            |row| row.get(0),
        )
        .map_err(|err| format!("Failed to fetch vault privacy: {err}"))?;

    Ok(vault_privacy)
}

#[allow(clippy::too_many_arguments)]
fn insert_changeset_node(
    tx: &Transaction,
    vault_id: &str,
    title: &str,
    summary: &str,
    detail: Option<&str>,
    node_type: &str,
    tags: Option<&Vec<String>>,
    session_key: Option<&redacted::SessionKey>,
) -> Result<String, String> {
    let parent_vault_id: Option<String> = tx
        .query_row(
            "SELECT parent_vault_id FROM vaults WHERE id = ?1 LIMIT 1;",
            [vault_id],
            |row| row.get(0),
        )
        .ok()
        .flatten();

    let (resolved_vault_id, resolved_sub_vault_id) = match parent_vault_id {
        Some(parent_id) => (parent_id, Some(vault_id.to_string())),
        None => (vault_id.to_string(), None),
    };

    let effective_privacy = resolve_effective_privacy(tx, &resolved_vault_id, None)?;
    let is_redacted = effective_privacy == "redacted";

    let encrypted_payload = if is_redacted {
        let key = session_key.ok_or_else(|| "VAULT_LOCKED".to_string())?;
        Some(redacted::encrypt_json(
            &redacted::NodeSecretPayload {
                title: title.to_string(),
                summary: summary.to_string(),
                detail: detail.map(String::from),
                source: Some("agent_extract".to_string()),
                source_type: Some("agent_extract".to_string()),
            },
            key,
        )?)
    } else {
        None
    };

    let stored_title = if is_redacted {
        "[REDACTED]".to_string()
    } else {
        title.to_string()
    };

    let stored_summary = if is_redacted {
        "[Metadata Locked]".to_string()
    } else {
        summary.to_string()
    };

    let node_id = crate::generate_id(tx, "node")?;
    let priority_json = priority::DEFAULT_PRIORITY_JSON;

    tx.execute(
        "INSERT INTO nodes (
            id, vault_id, sub_vault_id, node_type, title, summary, detail, source, source_type,
            privacy_tier, priority, meta, encrypted_payload
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'agent_extract', 'agent_extract', NULL, ?8, '{}', ?9);",
        params![
            node_id,
            resolved_vault_id,
            resolved_sub_vault_id,
            node_type,
            stored_title,
            stored_summary,
            if is_redacted { None } else { detail },
            priority_json,
            encrypted_payload
        ],
    )
    .map_err(|err| format!("Failed to insert changeset node: {err}"))?;

    if let Some(tag_list) = tags {
        for tag_name in tag_list {
            let clean_name = tag_name.trim();
            if clean_name.is_empty() {
                continue;
            }

            let tag_id = match tx.query_row(
                "SELECT id FROM tags WHERE name = ?1;",
                [clean_name],
                |row| row.get::<_, String>(0),
            ) {
                Ok(id) => id,
                Err(rusqlite::Error::QueryReturnedNoRows) => {
                    let new_id = crate::generate_id(tx, "tag")?;
                    tx.execute(
                        "INSERT INTO tags (id, name, color) VALUES (?1, ?2, NULL);",
                        params![new_id, clean_name],
                    )
                    .map_err(|err| format!("Failed inserting tag: {err}"))?;
                    new_id
                }
                Err(err) => return Err(format!("Failed querying tag: {err}")),
            };

            tx.execute(
                "INSERT OR IGNORE INTO node_tags (node_id, tag_id) VALUES (?1, ?2);",
                params![&node_id, &tag_id],
            )
            .map_err(|err| format!("Failed inserting node tag: {err}"))?;
        }
    }

    Ok(node_id)
}

#[allow(clippy::too_many_arguments)]
fn update_changeset_node(
    tx: &Transaction,
    node_id: &str,
    vault_id: &str,
    title: &str,
    summary: &str,
    detail: Option<&str>,
    node_type: &str,
    tags: Option<&Vec<String>>,
    session_key: Option<&redacted::SessionKey>,
) -> Result<(), String> {
    let parent_vault_id: Option<String> = tx
        .query_row(
            "SELECT parent_vault_id FROM vaults WHERE id = ?1 LIMIT 1;",
            [vault_id],
            |row| row.get(0),
        )
        .ok()
        .flatten();

    let (resolved_vault_id, resolved_sub_vault_id) = match parent_vault_id {
        Some(parent_id) => (parent_id, Some(vault_id.to_string())),
        None => (vault_id.to_string(), None),
    };

    let effective_privacy = resolve_effective_privacy(tx, &resolved_vault_id, None)?;
    let is_redacted = effective_privacy == "redacted";

    let encrypted_payload = if is_redacted {
        let key = session_key.ok_or_else(|| "VAULT_LOCKED".to_string())?;
        Some(redacted::encrypt_json(
            &redacted::NodeSecretPayload {
                title: title.to_string(),
                summary: summary.to_string(),
                detail: detail.map(String::from),
                source: Some("agent_extract".to_string()),
                source_type: Some("agent_extract".to_string()),
            },
            key,
        )?)
    } else {
        None
    };

    let stored_title = if is_redacted {
        "[REDACTED]".to_string()
    } else {
        title.to_string()
    };

    let stored_summary = if is_redacted {
        "[Metadata Locked]".to_string()
    } else {
        summary.to_string()
    };

    tx.execute(
        "UPDATE nodes
         SET vault_id = ?2,
             sub_vault_id = ?3,
             node_type = ?4,
             title = ?5,
             summary = ?6,
             detail = ?7,
             version = version + 1,
             updated_at = datetime('now'),
             encrypted_payload = ?8
         WHERE id = ?1 AND deleted_at IS NULL;",
        params![
            node_id,
            resolved_vault_id,
            resolved_sub_vault_id,
            node_type,
            stored_title,
            stored_summary,
            if is_redacted { None } else { detail },
            encrypted_payload
        ],
    )
    .map_err(|err| format!("Failed updating node: {err}"))?;

    tx.execute("DELETE FROM node_tags WHERE node_id = ?1;", [node_id])
        .map_err(|err| format!("Failed clearing node tags: {err}"))?;

    if let Some(tag_list) = tags {
        for tag_name in tag_list {
            let clean_name = tag_name.trim();
            if clean_name.is_empty() {
                continue;
            }

            let tag_id = match tx.query_row(
                "SELECT id FROM tags WHERE name = ?1;",
                [clean_name],
                |row| row.get::<_, String>(0),
            ) {
                Ok(id) => id,
                Err(rusqlite::Error::QueryReturnedNoRows) => {
                    let new_id = crate::generate_id(tx, "tag")?;
                    tx.execute(
                        "INSERT INTO tags (id, name, color) VALUES (?1, ?2, NULL);",
                        params![new_id, clean_name],
                    )
                    .map_err(|err| format!("Failed inserting tag: {err}"))?;
                    new_id
                }
                Err(err) => return Err(format!("Failed querying tag: {err}")),
            };

            tx.execute(
                "INSERT OR IGNORE INTO node_tags (node_id, tag_id) VALUES (?1, ?2);",
                params![&node_id, &tag_id],
            )
            .map_err(|err| format!("Failed inserting node tag: {err}"))?;
        }
    }

    Ok(())
}

pub fn commit_changeset_transaction(
    conn: &mut Connection,
    input: &ChangesetCommitInput,
    db_path: &Path,
    session_key: Option<redacted::SessionKey>,
) -> Result<bool, String> {
    // 1. Redacted Lock Check
    for item_action in &input.item_actions {
        if item_action.action == "accept" || item_action.action == "edit" {
            let item_row: Option<(String, String)> = conn
                .query_row(
                    "SELECT item_type, proposed_data FROM changeset_items WHERE id = ?1 LIMIT 1;",
                    [&item_action.item_id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .ok();

            if let Some((_, proposed_data)) = item_row {
                let proposed_json: serde_json::Value =
                    serde_json::from_str(&proposed_data).unwrap_or_else(|_| serde_json::json!({}));
                let target_vault_id = proposed_json
                    .get("vaultId")
                    .or_else(|| proposed_json.get("vault_id"))
                    .and_then(|v| v.as_str());

                if let Some(vid) = target_vault_id {
                    let target_tier: String = conn
                        .query_row(
                            "SELECT privacy_tier FROM vaults WHERE id = ?1 LIMIT 1;",
                            [vid],
                            |row| row.get(0),
                        )
                        .unwrap_or_else(|_| "open".to_string());

                    if target_tier == "redacted" && session_key.is_none() {
                        return Err("VAULT_LOCKED".to_string());
                    }
                }
            }
        }
    }

    // 2. Take pre-write database checkpoint
    if !input.item_actions.is_empty() {
        let _ = crate::minimal_pre_write_backup(conn, db_path, "changeset")?;
    }

    // 3. Begin atomic transaction scoping
    let tx = conn
        .transaction()
        .map_err(|err| format!("Failed starting changeset commit transaction: {err}"))?;

    let mut accepted_diff = 0i64;
    let mut dismissed_diff = 0i64;

    for item_action in &input.item_actions {
        match item_action.action.as_str() {
            "dismiss" => {
                tx.execute(
                    "UPDATE changeset_items SET status = 'dismissed', reviewed_at = datetime('now') WHERE id = ?1;",
                    [&item_action.item_id],
                )
                .map_err(|err| format!("Failed dismissing changeset item: {err}"))?;
                dismissed_diff += 1;
            }
            "accept" | "edit" => {
                let (item_type, proposed_data, target_node_id, merge_with_id): (
                    String,
                    String,
                    Option<String>,
                    Option<String>,
                ) = tx
                    .query_row(
                        "SELECT item_type, proposed_data, target_node_id, merge_with_id FROM changeset_items WHERE id = ?1 LIMIT 1;",
                        [&item_action.item_id],
                        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
                    )
                    .map_err(|err| format!("Failed fetching changeset item: {err}"))?;

                let parsed_props = if item_action.action == "edit" {
                    item_action
                        .edited_data
                        .clone()
                        .ok_or_else(|| "Missing edited data for edit action".to_string())?
                } else {
                    serde_json::from_str(&proposed_data)
                        .map_err(|err| format!("Failed to parse proposed properties: {err}"))?
                };

                let title = parsed_props
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let summary = parsed_props
                    .get("summary")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let detail = parsed_props.get("detail").and_then(|v| v.as_str());
                let node_type = parsed_props
                    .get("nodeType")
                    .or_else(|| parsed_props.get("node_type"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("concept");
                let vault_id = parsed_props
                    .get("vaultId")
                    .or_else(|| parsed_props.get("vault_id"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("vault_root_graph");
                let tags = parsed_props
                    .get("tags")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|val| val.as_str().map(String::from))
                            .collect::<Vec<String>>()
                    });

                match item_type.as_str() {
                    "add" => {
                        let _new_node_id = insert_changeset_node(
                            &tx,
                            vault_id,
                            title,
                            summary,
                            detail,
                            node_type,
                            tags.as_ref(),
                            session_key.as_ref(),
                        )?;
                    }
                    "update" => {
                        if let Some(ref nid) = target_node_id {
                            update_changeset_node(
                                &tx,
                                nid,
                                vault_id,
                                title,
                                summary,
                                detail,
                                node_type,
                                tags.as_ref(),
                                session_key.as_ref(),
                            )?;
                        }
                    }
                    "merge" => {
                        if let Some(ref mid) = merge_with_id {
                            // 1. Fetch current node details and tags
                            let (ex_detail, ex_vault_id, ex_title, ex_summary, ex_node_type): (
                                Option<String>,
                                String,
                                String,
                                String,
                                String,
                            ) = tx
                                .query_row(
                                    "SELECT detail, vault_id, title, summary, node_type FROM nodes WHERE id = ?1;",
                                    [mid],
                                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
                                )
                                .map_err(|err| format!("Failed fetching node for merge: {err}"))?;

                            // 2. Append details
                            let mut merged_detail = ex_detail.unwrap_or_default();
                            if let Some(new_det) = detail {
                                if !new_det.trim().is_empty() {
                                    if !merged_detail.is_empty() {
                                        merged_detail.push_str("\n\n");
                                    }
                                    merged_detail.push_str(new_det.trim());
                                }
                            }

                            // 3. Union tags
                            let mut merged_tags = HashSet::new();
                            let mut stmt = tx
                                .prepare("SELECT t.name FROM node_tags nt JOIN tags t ON nt.tag_id = t.id WHERE nt.node_id = ?1;")
                                .map_err(|err| format!("Failed querying current tags: {err}"))?;
                            let rows = stmt
                                .query_map([mid], |row| row.get::<_, String>(0))
                                .map_err(|err| format!("Failed fetching current tags: {err}"))?;
                            for r in rows.flatten() {
                                merged_tags.insert(r);
                            }
                            if let Some(ref new_tags) = tags {
                                for t in new_tags {
                                    merged_tags.insert(t.clone());
                                }
                            }
                            let merged_tags_vec: Vec<String> = merged_tags.into_iter().collect();

                            update_changeset_node(
                                &tx,
                                mid,
                                &ex_vault_id,
                                &ex_title,
                                &ex_summary,
                                if merged_detail.is_empty() {
                                    None
                                } else {
                                    Some(&merged_detail)
                                },
                                &ex_node_type,
                                Some(&merged_tags_vec),
                                session_key.as_ref(),
                            )?;
                        }
                    }
                    "delete" => {
                        if let Some(ref nid) = target_node_id {
                            tx.execute(
                                "UPDATE nodes SET deleted_at = datetime('now'), updated_at = datetime('now') WHERE id = ?1;",
                                [nid],
                            )
                            .map_err(|err| format!("Failed soft deleting node: {err}"))?;
                        }
                    }
                    "repoint_door" | "orphan_alert" => {
                        let door_id: Option<String> = tx
                            .query_row(
                                "SELECT door_id FROM changeset_items WHERE id = ?1 LIMIT 1;",
                                [&item_action.item_id],
                                |row| row.get(0),
                            )
                            .ok()
                            .flatten();

                        if let Some(ref did) = door_id {
                            if let Some(ref nid) = target_node_id {
                                tx.execute(
                                    "UPDATE doors SET target_node_id = ?1, status = 'active', updated_at = datetime('now') WHERE id = ?2;",
                                    params![nid, did],
                                )
                                .map_err(|err| format!("Failed repointing door: {err}"))?;

                                // Backlink triggers will auto-sync backlinks
                            }
                        }
                    }
                    _ => {}
                }

                tx.execute(
                    "UPDATE changeset_items SET status = 'accepted', reviewed_at = datetime('now') WHERE id = ?1;",
                    [&item_action.item_id],
                )
                .map_err(|err| format!("Failed accepting changeset item: {err}"))?;
                accepted_diff += 1;
            }
            _ => {}
        }
    }

    // 4. Update parent changeset counts and status
    tx.execute(
        "UPDATE changesets
         SET accepted_count = accepted_count + ?2,
             dismissed_count = dismissed_count + ?3
         WHERE id = ?1;",
        params![input.changeset_id, accepted_diff, dismissed_diff],
    )
    .map_err(|err| format!("Failed updating parent changeset counts: {err}"))?;

    let (item_count, accepted_count, dismissed_count): (i64, i64, i64) = tx
        .query_row(
            "SELECT item_count, accepted_count, dismissed_count FROM changesets WHERE id = ?1 LIMIT 1;",
            [&input.changeset_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|err| format!("Failed fetching resolved counts for changeset: {err}"))?;

    let resolved_status = if accepted_count + dismissed_count >= item_count {
        if accepted_count == item_count {
            "accepted"
        } else if dismissed_count == item_count {
            "dismissed"
        } else {
            "partial"
        }
    } else {
        "pending"
    };

    tx.execute(
        "UPDATE changesets
         SET status = ?2,
             reviewed_at = datetime('now')
         WHERE id = ?1;",
        params![input.changeset_id, resolved_status],
    )
    .map_err(|err| format!("Failed final status update on parent changeset: {err}"))?;

    tx.commit()
        .map_err(|err| format!("Failed committing changeset transaction: {err}"))?;

    Ok(true)
}
