use crate::error::AppResult;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub id: String,
    pub display_name: String,
    #[serde(with = "hex_serde")]
    pub identity_key: Vec<u8>,
    pub safety_number: Option<String>,
    pub verified: bool,
    pub created_at: String,
}

/// Hex serializer for identity_key so frontend gets hex strings
mod hex_serde {
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(bytes))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        hex::decode(&s).map_err(serde::de::Error::custom)
    }
}

pub fn insert_contact(conn: &Connection, contact: &Contact) -> AppResult<()> {
    conn.execute(
        r#"INSERT INTO contacts (id, display_name, identity_key, safety_number, verified)
           VALUES (?1, ?2, ?3, ?4, ?5)
           ON CONFLICT(id) DO UPDATE SET
               display_name = excluded.display_name,
               updated_at = datetime('now')"#,
        params![
            contact.id,
            contact.display_name,
            contact.identity_key,
            contact.safety_number,
            contact.verified as i32,
        ],
    )?;
    Ok(())
}

pub fn get_all_contacts(conn: &Connection) -> AppResult<Vec<Contact>> {
    let mut stmt = conn.prepare(
        r#"SELECT id, display_name, identity_key, safety_number, verified, created_at
           FROM contacts ORDER BY display_name"#,
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(Contact {
            id: row.get(0)?,
            display_name: row.get(1)?,
            identity_key: row.get(2)?,
            safety_number: row.get(3)?,
            verified: row.get::<_, i32>(4)? != 0,
            created_at: row.get(5)?,
        })
    })?;

    let mut contacts = Vec::new();
    for row in rows {
        contacts.push(row?);
    }
    Ok(contacts)
}

pub fn get_contact(conn: &Connection, id: &str) -> AppResult<Option<Contact>> {
    let mut stmt = conn.prepare(
        r#"SELECT id, display_name, identity_key, safety_number, verified, created_at
           FROM contacts WHERE id = ?1"#,
    )?;

    let result = stmt
        .query_row(params![id], |row| {
            Ok(Contact {
                id: row.get(0)?,
                display_name: row.get(1)?,
                identity_key: row.get(2)?,
                safety_number: row.get(3)?,
                verified: row.get::<_, i32>(4)? != 0,
                created_at: row.get(5)?,
            })
        })
        .ok();

    Ok(result)
}

pub fn delete_contact(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM contacts WHERE id = ?1", params![id])?;
    Ok(())
}
