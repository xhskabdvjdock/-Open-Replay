use anyhow::{Context, Result};
use rusqlite::{Connection, params};
use std::path::PathBuf;
use tracing::info;

/// SQLite database for clip metadata
pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(db_path: &PathBuf) -> Result<Self> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(db_path)
            .with_context(|| format!("Opening database at {:?}", db_path))?;

        // Enable WAL mode for better concurrent performance
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;

        let db = Self { conn };
        db.initialize_schema()?;
        info!("Database opened at {:?}", db_path);
        Ok(db)
    }

    fn initialize_schema(&self) -> Result<()> {
        self.conn.execute_batch(r#"
            CREATE TABLE IF NOT EXISTS clips (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                file_path TEXT NOT NULL UNIQUE,
                thumbnail_path TEXT,
                created_at TEXT NOT NULL,
                duration_ms INTEGER,
                width INTEGER,
                height INTEGER,
                fps REAL,
                file_size INTEGER,
                has_audio INTEGER DEFAULT 0,
                codec TEXT DEFAULT 'h264',
                notes TEXT
            );

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_clips_created_at ON clips(created_at DESC);
            CREATE INDEX IF NOT EXISTS idx_clips_name ON clips(name);
        "#)?;
        Ok(())
    }

    pub fn insert_clip(&self, clip: &ClipRecord) -> Result<()> {
        self.conn.execute(
            r#"INSERT OR REPLACE INTO clips
               (id, name, file_path, thumbnail_path, created_at, duration_ms,
                width, height, fps, file_size, has_audio, codec, notes)
               VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)"#,
            params![
                clip.id,
                clip.name,
                clip.file_path,
                clip.thumbnail_path,
                clip.created_at,
                clip.duration_ms,
                clip.width,
                clip.height,
                clip.fps,
                clip.file_size,
                clip.has_audio as i32,
                clip.codec,
                clip.notes,
            ],
        )?;
        Ok(())
    }

    pub fn get_clip(&self, id: &str) -> Result<Option<ClipRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id,name,file_path,thumbnail_path,created_at,duration_ms,width,height,fps,file_size,has_audio,codec,notes FROM clips WHERE id=?1"
        )?;
        let mut rows = stmt.query_map(params![id], map_clip_row)?;
        Ok(rows.next().transpose()?)
    }

    pub fn get_all_clips(&self, sort_field: &str, sort_order: &str) -> Result<Vec<ClipRecord>> {
        let safe_field = match sort_field {
            "name" => "name",
            "duration" => "duration_ms",
            "size" => "file_size",
            _ => "created_at",
        };
        let safe_order = if sort_order == "asc" { "ASC" } else { "DESC" };
        let query = format!(
            "SELECT id,name,file_path,thumbnail_path,created_at,duration_ms,width,height,fps,file_size,has_audio,codec,notes FROM clips ORDER BY {} {}",
            safe_field, safe_order
        );
        let mut stmt = self.conn.prepare(&query)?;
        let rows = stmt.query_map([], map_clip_row)?;
        rows.collect::<rusqlite::Result<Vec<_>>>().map_err(Into::into)
    }

    pub fn search_clips(&self, query: &str) -> Result<Vec<ClipRecord>> {
        let pattern = format!("%{}%", query);
        let mut stmt = self.conn.prepare(
            "SELECT id,name,file_path,thumbnail_path,created_at,duration_ms,width,height,fps,file_size,has_audio,codec,notes FROM clips WHERE name LIKE ?1 ORDER BY created_at DESC"
        )?;
        let rows = stmt.query_map(params![pattern], map_clip_row)?;
        rows.collect::<rusqlite::Result<Vec<_>>>().map_err(Into::into)
    }

    pub fn delete_clip(&self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM clips WHERE id=?1", params![id])?;
        Ok(())
    }

    pub fn rename_clip(&self, id: &str, new_name: &str) -> Result<()> {
        self.conn.execute("UPDATE clips SET name=?1 WHERE id=?2", params![new_name, id])?;
        Ok(())
    }

    pub fn update_thumbnail(&self, id: &str, thumbnail_path: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE clips SET thumbnail_path=?1 WHERE id=?2",
            params![thumbnail_path, id],
        )?;
        Ok(())
    }

    pub fn clip_exists_by_path(&self, file_path: &str) -> Result<bool> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM clips WHERE file_path=?1",
            params![file_path],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn get_all_clip_paths(&self) -> Result<Vec<(String, String)>> {
        let mut stmt = self.conn.prepare("SELECT id, file_path FROM clips")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>().map_err(Into::into)
    }

    pub fn remove_orphan(&self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM clips WHERE id=?1", params![id])?;
        Ok(())
    }
}

fn map_clip_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ClipRecord> {
    Ok(ClipRecord {
        id: row.get(0)?,
        name: row.get(1)?,
        file_path: row.get(2)?,
        thumbnail_path: row.get(3)?,
        created_at: row.get(4)?,
        duration_ms: row.get(5)?,
        width: row.get(6)?,
        height: row.get(7)?,
        fps: row.get(8)?,
        file_size: row.get(9)?,
        has_audio: row.get::<_, i32>(10)? != 0,
        codec: row.get(11)?,
        notes: row.get(12)?,
    })
}

#[derive(Debug, Clone)]
pub struct ClipRecord {
    pub id: String,
    pub name: String,
    pub file_path: String,
    pub thumbnail_path: Option<String>,
    pub created_at: String,
    pub duration_ms: Option<i64>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub fps: Option<f64>,
    pub file_size: Option<i64>,
    pub has_audio: bool,
    pub codec: Option<String>,
    pub notes: Option<String>,
}
