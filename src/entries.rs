use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Pending,
    Approved,
    Denied,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryMeta {
    pub name: String,
    pub date: String,
    #[serde(default)]
    pub website: String,
    #[serde(default)]
    pub drawing: String,
    #[serde(default)]
    pub voice_note: String,
    pub status: Status,
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub id: String,
    pub meta: EntryMeta,
    pub body: String,
}

impl Entry {
    /// Parse an entry from file contents. `id` is derived from the filename.
    pub fn parse(id: &str, contents: &str) -> Result<Self, String> {
        let contents = contents.trim();
        if !contents.starts_with("+++") {
            return Err("missing opening +++".into());
        }
        let rest = &contents[3..];
        let end = rest.find("+++").ok_or("missing closing +++")?;
        let frontmatter = &rest[..end];
        let body = rest[end + 3..].trim_start_matches('\n').to_string();
        let meta: EntryMeta =
            toml::from_str(frontmatter).map_err(|e| format!("bad frontmatter: {e}"))?;
        Ok(Entry {
            id: id.to_string(),
            meta,
            body,
        })
    }

    /// Serialize entry back to file format.
    pub fn to_file_contents(&self) -> String {
        let frontmatter = toml::to_string_pretty(&self.meta).unwrap();
        format!("+++\n{frontmatter}+++\n{}", self.body)
    }
}

/// Read all entries from the given directory.
pub fn read_entries(dir: &Path) -> Vec<Entry> {
    let mut entries = Vec::new();
    let read_dir = match std::fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return entries,
    };
    for item in read_dir {
        let Ok(item) = item else { continue };
        let path = item.path();
        if path.extension().and_then(|e| e.to_str()) != Some("txt") {
            continue;
        }
        let id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        let Ok(contents) = std::fs::read_to_string(&path) else {
            continue;
        };
        if let Ok(entry) = Entry::parse(&id, &contents) {
            entries.push(entry);
        }
    }
    entries.sort_by(|a, b| b.meta.date.cmp(&a.meta.date));
    entries
}

/// Read entries filtered by status.
pub fn read_by_status(dir: &Path, status: Status) -> Vec<Entry> {
    read_entries(dir)
        .into_iter()
        .filter(|e| e.meta.status == status)
        .collect()
}

/// Read approved entries only.
pub fn read_approved(dir: &Path) -> Vec<Entry> {
    read_by_status(dir, Status::Approved)
}

/// Find an entry file by short ID prefix and update its status.
pub fn set_status(dir: &Path, short_id: &str, status: Status) -> Result<String, String> {
    let read_dir = std::fs::read_dir(dir).map_err(|e| e.to_string())?;
    for item in read_dir {
        let Ok(item) = item else { continue };
        let path = item.path();
        let fname = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if fname.contains(short_id) && fname.ends_with(".txt") {
            let contents = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
            let id = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            let mut entry = Entry::parse(&id, &contents)?;
            entry.meta.status = status;
            std::fs::write(&path, entry.to_file_contents()).map_err(|e| e.to_string())?;
            return Ok(entry.meta.name.clone());
        }
    }
    Err("Not found.".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_entry() {
        let contents = r#"+++
name = "alice"
date = "2026-04-09"
website = "https://example.com"
status = "approved"
+++
Hello world!"#;
        let entry = Entry::parse("2026-04-09-abcd1234", contents).unwrap();
        assert_eq!(entry.id, "2026-04-09-abcd1234");
        assert_eq!(entry.meta.name, "alice");
        assert_eq!(entry.meta.date, "2026-04-09");
        assert_eq!(entry.meta.website, "https://example.com");
        assert_eq!(entry.meta.status, Status::Approved);
        assert_eq!(entry.body, "Hello world!");
    }

    #[test]
    fn test_parse_entry_no_website() {
        let contents = r#"+++
name = "bob"
date = "2026-04-09"
status = "pending"
+++
Just a message."#;
        let entry = Entry::parse("2026-04-09-ef567890", contents).unwrap();
        assert_eq!(entry.meta.website, "");
        assert_eq!(entry.meta.status, Status::Pending);
    }

    #[test]
    fn test_parse_entry_with_html() {
        let contents = r#"+++
name = "carol"
date = "2026-04-09"
status = "approved"
+++
<b>Bold</b> and <em>italic</em>.

<em>-- llywelwyn</em>: Thanks!"#;
        let entry = Entry::parse("2026-04-09-11223344", contents).unwrap();
        assert!(entry.body.contains("<b>Bold</b>"));
        assert!(entry.body.contains("<em>-- llywelwyn</em>"));
    }

    #[test]
    fn test_roundtrip() {
        let contents = r#"+++
name = "alice"
date = "2026-04-09"
website = "https://example.com"
status = "approved"
+++
Hello world!"#;
        let entry = Entry::parse("test", contents).unwrap();
        let serialized = entry.to_file_contents();
        let reparsed = Entry::parse("test", &serialized).unwrap();
        assert_eq!(entry.meta.name, reparsed.meta.name);
        assert_eq!(entry.body, reparsed.body);
    }

    #[test]
    fn test_parse_missing_frontmatter() {
        let result = Entry::parse("x", "no frontmatter here");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_entry_with_drawing() {
        let contents = r#"+++
name = "alice"
date = "2026-04-09"
status = "approved"
drawing = "abc123.png"
+++
Hello!"#;
        let entry = Entry::parse("test", contents).unwrap();
        assert_eq!(entry.meta.drawing, "abc123.png");
    }

    #[test]
    fn test_parse_entry_without_drawing() {
        let contents = r#"+++
name = "bob"
date = "2026-04-09"
status = "pending"
+++
Hi!"#;
        let entry = Entry::parse("test", contents).unwrap();
        assert_eq!(entry.meta.drawing, "");
    }

    #[test]
    fn test_roundtrip_with_drawing() {
        let contents = r#"+++
name = "alice"
date = "2026-04-09"
status = "approved"
drawing = "abc123.png"
+++
Hello!"#;
        let entry = Entry::parse("test", contents).unwrap();
        let serialized = entry.to_file_contents();
        let reparsed = Entry::parse("test", &serialized).unwrap();
        assert_eq!(reparsed.meta.drawing, "abc123.png");
    }

    #[test]
    fn test_parse_entry_with_voice_note() {
        let contents = r#"+++
name = "alice"
date = "2026-04-10"
status = "approved"
voice_note = "1744300800_abcd1234.webm"
+++
Hello!"#;
        let entry = Entry::parse("test", contents).unwrap();
        assert_eq!(entry.meta.voice_note, "1744300800_abcd1234.webm");
    }

    #[test]
    fn test_parse_entry_without_voice_note() {
        let contents = r#"+++
name = "bob"
date = "2026-04-10"
status = "pending"
+++
Hi!"#;
        let entry = Entry::parse("test", contents).unwrap();
        assert_eq!(entry.meta.voice_note, "");
    }

    #[test]
    fn test_read_by_status() {
        let dir = tempfile::tempdir().unwrap();
        let approved = "+++\nname = \"a\"\ndate = \"2026-04-10\"\nstatus = \"approved\"\n+++\nhi";
        let pending = "+++\nname = \"b\"\ndate = \"2026-04-10\"\nstatus = \"pending\"\n+++\nhi";
        let denied = "+++\nname = \"c\"\ndate = \"2026-04-10\"\nstatus = \"denied\"\n+++\nhi";
        std::fs::write(dir.path().join("1_aaa.txt"), approved).unwrap();
        std::fs::write(dir.path().join("2_bbb.txt"), pending).unwrap();
        std::fs::write(dir.path().join("3_ccc.txt"), denied).unwrap();

        assert_eq!(read_by_status(dir.path(), Status::Approved).len(), 1);
        assert_eq!(read_by_status(dir.path(), Status::Pending).len(), 1);
        assert_eq!(read_by_status(dir.path(), Status::Denied).len(), 1);
        assert_eq!(read_by_status(dir.path(), Status::Approved)[0].meta.name, "a");
    }

    #[test]
    fn test_roundtrip_with_voice_note() {
        let contents = r#"+++
name = "alice"
date = "2026-04-10"
status = "approved"
voice_note = "1744300800_abcd1234.webm"
+++
Hello!"#;
        let entry = Entry::parse("test", contents).unwrap();
        let serialized = entry.to_file_contents();
        let reparsed = Entry::parse("test", &serialized).unwrap();
        assert_eq!(reparsed.meta.voice_note, "1744300800_abcd1234.webm");
    }
}
