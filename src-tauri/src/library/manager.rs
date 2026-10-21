use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{info, warn, error};
use uuid::Uuid;
use base64::Engine;

use crate::storage::database::{Database, ClipRecord};

/// Clip info returned to frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipInfo {
    pub id: String,
    pub name: String,
    pub file_path: String,
    pub thumbnail_path: Option<String>,
    pub created_at: String,
    pub duration_ms: i64,
    pub width: i32,
    pub height: i32,
    pub fps: f64,
    pub file_size: i64,
    pub has_audio: bool,
    pub codec: String,
    pub notes: Option<String>,
}

impl From<ClipRecord> for ClipInfo {
    fn from(r: ClipRecord) -> Self {
        Self {
            id: r.id,
            name: r.name,
            file_path: r.file_path,
            thumbnail_path: r.thumbnail_path,
            created_at: r.created_at,
            duration_ms: r.duration_ms.unwrap_or(0),
            width: r.width.unwrap_or(0),
            height: r.height.unwrap_or(0),
            fps: r.fps.unwrap_or(0.0),
            file_size: r.file_size.unwrap_or(0),
            has_audio: r.has_audio,
            codec: r.codec.unwrap_or_else(|| "h264".into()),
            notes: r.notes,
        }
    }
}

/// Manages the clip library: database + filesystem
pub struct Library {
    db: Database,
    clips_dir: PathBuf,
    thumbnails_dir: PathBuf,
}

impl Library {
    pub fn new(data_dir: &PathBuf) -> Result<Self> {
        let clips_dir = data_dir.join("clips");
        let thumbnails_dir = data_dir.join("thumbnails");
        let db_path = data_dir.join("library.db");

        std::fs::create_dir_all(&clips_dir)?;
        std::fs::create_dir_all(&thumbnails_dir)?;

        let db = Database::open(&db_path)?;

        let lib = Self { db, clips_dir, thumbnails_dir };
        lib.scan_and_reconcile()?;
        Ok(lib)
    }

    /// Reconcile database with actual files on disk
    pub fn scan_and_reconcile(&self) -> Result<()> {
        // Remove db records for files that no longer exist
        let paths = self.db.get_all_clip_paths()?;
        for (id, path) in &paths {
            if !Path::new(path).exists() {
                warn!("Clip file missing, removing from db: {}", path);
                self.db.remove_orphan(id)?;
            }
        }

        // Add any .mp4 files in clips_dir that aren't in db
        if let Ok(entries) = std::fs::read_dir(&self.clips_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("mp4") {
                    let path_str = path.to_string_lossy().into_owned();
                    if !self.db.clip_exists_by_path(&path_str)? {
                        info!("Discovered untracked clip: {}", path_str);
                        if let Err(e) = self.import_external_file(&path) {
                            warn!("Failed to import discovered clip {:?}: {}", path, e);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn import_external_file(&self, path: &Path) -> Result<()> {
        let id = Uuid::new_v4().to_string();
        let name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled")
            .to_string();
        let file_size = std::fs::metadata(path)?.len() as i64;
        let created_at = Utc::now().to_rfc3339();
        let path_str = path.to_string_lossy().into_owned();

        let record = ClipRecord {
            id,
            name,
            file_path: path_str,
            thumbnail_path: None,
            created_at,
            duration_ms: None,
            width: None,
            height: None,
            fps: None,
            file_size: Some(file_size),
            has_audio: false,
            codec: Some("h264".into()),
            notes: None,
        };
        self.db.insert_clip(&record)?;
        Ok(())
    }

    /// Register a newly saved clip
    pub fn add_clip(
        &self,
        file_path: &Path,
        duration_ms: i64,
        width: i32,
        height: i32,
        fps: f64,
        has_audio: bool,
    ) -> Result<ClipInfo> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let name = format!("Replay {}", now.format("%Y-%m-%d %H-%M-%S"));
        let file_size = std::fs::metadata(file_path)
            .map(|m| m.len() as i64)
            .unwrap_or(0);

        // Generate thumbnail
        let thumbnail_path = self
            .generate_thumbnail(&id, file_path)
            .map_err(|e| { warn!("Thumbnail generation failed: {}", e); e })
            .ok()
            .map(|p| p.to_string_lossy().into_owned());

        let record = ClipRecord {
            id: id.clone(),
            name: name.clone(),
            file_path: file_path.to_string_lossy().into_owned(),
            thumbnail_path: thumbnail_path.clone(),
            created_at: now.to_rfc3339(),
            duration_ms: Some(duration_ms),
            width: Some(width),
            height: Some(height),
            fps: Some(fps),
            file_size: Some(file_size),
            has_audio,
            codec: Some("h264".into()),
            notes: None,
        };
        self.db.insert_clip(&record)?;
        info!("Clip added to library: {} -> {:?}", id, file_path);
        Ok(ClipInfo::from(record))
    }

    fn generate_thumbnail(&self, clip_id: &str, video_path: &Path) -> Result<PathBuf> {
        // On Windows, use Windows Media Foundation to extract first keyframe
        // For now we create a placeholder dark thumbnail with the clip id
        // TODO: implement actual frame extraction with MF Source Reader
        let thumb_path = self.thumbnails_dir.join(format!("{}.jpg", clip_id));
        
        // Create a minimal 320x180 dark gray JPEG as placeholder
        let img = image::RgbImage::from_fn(320, 180, |x, y| {
            let val = if (x / 40 + y / 30) % 2 == 0 { 30u8 } else { 20u8 };
            image::Rgb([val, val, val])
        });
        img.save(&thumb_path).context("Saving thumbnail")?;
        Ok(thumb_path)
    }

    pub fn get_clip(&self, id: &str) -> Result<Option<ClipInfo>> {
        Ok(self.db.get_clip(id)?.map(ClipInfo::from))
    }

    pub fn get_clips(&self, sort_field: &str, sort_order: &str, search: Option<&str>) -> Result<Vec<ClipInfo>> {
        let records = if let Some(q) = search.filter(|s| !s.is_empty()) {
            self.db.search_clips(q)?
        } else {
            self.db.get_all_clips(sort_field, sort_order)?
        };
        Ok(records.into_iter().map(ClipInfo::from).collect())
    }

    pub fn delete_clip(&self, id: &str) -> Result<()> {
        if let Some(clip) = self.db.get_clip(id)? {
            // Delete video file
            if let Err(e) = std::fs::remove_file(&clip.file_path) {
                warn!("Could not delete clip file {:?}: {}", clip.file_path, e);
            }
            // Delete thumbnail
            if let Some(thumb) = &clip.thumbnail_path {
                let _ = std::fs::remove_file(thumb);
            }
        }
        self.db.delete_clip(id)?;
        info!("Clip {} deleted", id);
        Ok(())
    }

    pub fn rename_clip(&self, id: &str, new_name: &str) -> Result<()> {
        if new_name.trim().is_empty() {
            anyhow::bail!("Clip name cannot be empty");
        }
        self.db.rename_clip(id, new_name.trim())?;
        Ok(())
    }

    pub fn get_clip_thumbnail_b64(&self, id: &str) -> Result<String> {
        let clip = self.db.get_clip(id)?
            .ok_or_else(|| anyhow::anyhow!("Clip not found: {}", id))?;
        
        let thumb_path = clip.thumbnail_path
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("No thumbnail for clip {}", id))?;
        
        let data = std::fs::read(thumb_path)
            .with_context(|| format!("Reading thumbnail {}", thumb_path))?;
        
        Ok(base64::engine::general_purpose::STANDARD.encode(&data))
    }

    pub fn clips_dir(&self) -> &PathBuf {
        &self.clips_dir
    }

    pub fn next_clip_path(&self) -> PathBuf {
        let now = Utc::now();
        let filename = format!("replay_{}.mp4", now.format("%Y%m%d_%H%M%S"));
        self.clips_dir.join(filename)
    }

    pub fn get_disk_usage(&self) -> Result<(u64, u64)> {
        let mut total: u64 = 0;
        if let Ok(entries) = std::fs::read_dir(&self.clips_dir) {
            for entry in entries.flatten() {
                if let Ok(meta) = entry.metadata() {
                    total += meta.len();
                }
            }
        }
        // Free space on clips volume
        let free = get_disk_free(&self.clips_dir);
        Ok((total, free))
    }
}

#[cfg(windows)]
fn get_disk_free(path: &Path) -> u64 {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    let wide: Vec<u16> = OsStr::new(path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        let mut free_bytes: u64 = 0;
        let mut total_bytes: u64 = 0;
        let mut total_free: u64 = 0;
        if windows::Win32::Storage::FileSystem::GetDiskFreeSpaceExW(
            windows::core::PCWSTR(wide.as_ptr()),
            Some(&mut free_bytes),
            Some(&mut total_bytes),
            Some(&mut total_free),
        ).is_ok() {
            return free_bytes;
        }
    }
    0
}

#[cfg(not(windows))]
fn get_disk_free(_path: &Path) -> u64 {
    0
}
