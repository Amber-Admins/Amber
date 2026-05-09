use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub id: String,
    pub role: String,
    pub content: String,
    pub created_at: String,
}

pub fn get_chat_history(db: &Connection) -> Result<Vec<ChatMessage>, crate::AppError> {
    let mut statement = db
        .prepare(
            "SELECT id, role, content, created_at
             FROM session_messages
             ORDER BY created_at ASC;",
        )
        .map_err(|err| format!("Failed preparing chat history query: {err}"))?;

    let rows = statement
        .query_map([], |row| {
            Ok(ChatMessage {
                id: row.get(0)?,
                role: row.get(1)?,
                content: row.get(2)?,
                created_at: row.get(3)?,
            })
        })
        .map_err(|err| format!("Failed querying chat history: {err}"))?;

    let mut messages = Vec::new();
    for row in rows {
        messages.push(row.map_err(|err| format!("Failed decoding chat history row: {err}"))?);
    }
    Ok(messages)
}

pub fn append_message(
    db: &Connection,
    id: String,
    role: String,
    content: String,
) -> Result<(), crate::AppError> {
    db.execute(
        "INSERT INTO session_messages (id, role, content) VALUES (?1, ?2, ?3);",
        params![id, role, content],
    )
    .map_err(|err| format!("Failed appending chat message: {err}"))?;

    Ok(())
}
