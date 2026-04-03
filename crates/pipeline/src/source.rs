//! ERA-backed asset resolution using a [`FileProvider`] for I/O.
//!
//! [`AssetSource`] owns the ERA resolution logic (load order, index lookups,
//! decompression) while delegating raw file reads to a generic
//! [`FileProvider`].  The CLI uses [`StdFileProvider`] (backed by
//! `std::fs::read`); an engine could swap in memory-mapped I/O.
//!
//! The "last loaded wins" priority rule matches the engine's
//! `BFileManager::resolveFile` (confirmed in IDA at 0x140807090).
//!
//! ## Override layer
//!
//! An optional **override directory** sits in front of the ERA archives.
//! When set, [`AssetSource`] checks the override dir first on every resolve.
//! Write-back methods serialize files into the override dir, organised by
//! source ERA label so the tree mirrors the archive structure:
//!
//! ```text
//! overrides/
//!   root.era/
//!     data\objects.xml.xmb
//!     art\unsc\vehicle\warthog_01\warthog_01.vis.xmb
//!   scenarioshared.era/
//!     ...
//! ```

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use assets::{AssetResolver, FileProvider};

/// A loaded ERA archive with a pre-built filename → entry index map.
///
/// `D` is the data handle from [`FileProvider::Data`] (e.g. `Vec<u8>`, `Mmap`).
/// `std::io::Cursor<D>` owns the data and provides `Read + Seek` — no
/// self-referential borrows needed.
struct LoadedArchive<D: AsRef<[u8]>> {
    reader: era::Reader<era::crypto::decrypt::Reader<std::io::Cursor<D>>>,
    /// Archive label for diagnostics (e.g. "root.era").
    label: String,
    /// Normalised filename → entry index.
    index: HashMap<String, usize>,
}

/// Which ERA archive a file was resolved from.
#[derive(Debug, Clone)]
pub struct Provenance {
    /// ERA label (e.g. `"root.era"`).
    pub era_label: String,
    /// Normalised game path (e.g. `"data\\objects.xml.xmb"`).
    pub game_path: String,
}

/// Unified asset source backed by one or more ERA archives, with an
/// optional filesystem override layer for read/write support.
///
/// Generic over a [`FileProvider`] that supplies the raw ERA file bytes.
/// Resolution follows the engine rule: **highest load order (last loaded) wins**.
/// When an override directory is set, it is checked **before** any ERA.
pub struct AssetSource<F: FileProvider> {
    provider: F,
    archives: Vec<LoadedArchive<F::Data>>,
    /// Optional override directory for read/write support.
    override_dir: Option<PathBuf>,
}

impl<F: FileProvider> AssetSource<F> {
    /// Create an empty asset source with the given file provider.
    pub fn with_provider(provider: F) -> Self {
        Self {
            provider,
            archives: Vec::new(),
            override_dir: None,
        }
    }

    /// Set the override directory for read/write support.
    ///
    /// When set, [`resolve`](AssetResolver::resolve) checks this directory
    /// first, and [`write_file`] writes into it.
    pub fn set_override_dir(&mut self, path: impl Into<PathBuf>) {
        self.override_dir = Some(path.into());
    }

    /// Return the current override directory, if set.
    pub fn override_dir(&self) -> Option<&Path> {
        self.override_dir.as_deref()
    }

    /// Open an encrypted ERA archive and add it to the source.
    pub fn add_era(&mut self, path: &str) -> Result<usize, String> {
        let data = self
            .provider
            .open(path)
            .ok_or_else(|| format!("File not found: {path}"))?;

        // Cursor<D> owns the data and provides Read + Seek.
        let cursor = std::io::Cursor::new(data);
        let reader = era::Reader::from_encrypted(cursor, era::TeaKeys::default_archive_keys())
            .map_err(|e| format!("Failed to parse ERA archive {path}: {e}"))?;

        let mut index = HashMap::with_capacity(reader.entries().len());
        for (i, entry) in reader.entries().iter().enumerate() {
            if let Some(name) = &entry.filename {
                let key = normalise_path(name);
                index.insert(key, i);
            }
        }

        let entry_count = reader.entries().len();
        let label = std::path::Path::new(path)
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string());

        self.archives.push(LoadedArchive {
            reader,
            label,
            index,
        });

        Ok(entry_count)
    }

    /// Resolve a file by its real path, with fallback suffixes.
    ///
    /// Tries the exact path first, then appends each suffix in order.
    /// For example, `resolve_with_fallback("data\\objects.xml", &[".xmb"])`
    /// tries `data\objects.xml` then `data\objects.xml.xmb`.
    ///
    /// This is generic so we can add more compiled formats later.
    pub fn resolve_with_fallback(&mut self, path: &str, suffixes: &[&str]) -> Option<Vec<u8>> {
        if let Some(data) = self.resolve_exact(path) {
            return Some(data);
        }
        for suffix in suffixes {
            let fallback = format!("{path}{suffix}");
            if let Some(data) = self.resolve_exact(&fallback) {
                return Some(data);
            }
        }
        None
    }

    /// Resolve a data file by its real path (e.g. `data\objects.xml`,
    /// `art\foo.vis`), trying the compiled `.xmb` variant as fallback.
    pub fn resolve_data(&mut self, path: &str) -> Option<Vec<u8>> {
        self.resolve_with_fallback(path, &[".xmb"])
    }

    /// Read and parse an XMB document by its real path, with `.xmb` fallback.
    ///
    /// Pass the path you intend (e.g. `data\objects.xml`, `art\foo.vis`).
    /// The file bytes are parsed as XMB binary regardless of which variant
    /// was found — plain XML text is not yet supported.
    pub fn read_xmb(&mut self, path: &str) -> Option<xmb::Document> {
        let data = self.resolve_data(path)?;
        xmb::Reader::read(&data).ok()
    }

    /// Return a summary of loaded archives (for diagnostics).
    pub fn summary(&self) -> Vec<(&str, usize)> {
        self.archives
            .iter()
            .map(|a| (a.label.as_str(), a.index.len()))
            .collect()
    }

    /// Return all filenames per archive (label → sorted file list).
    pub fn files_per_archive(&self) -> Vec<(&str, Vec<&str>)> {
        self.archives
            .iter()
            .map(|a| {
                let mut files: Vec<&str> = a.index.keys().map(|k| k.as_str()).collect();
                files.sort();
                (a.label.as_str(), files)
            })
            .collect()
    }

    // ── Provenance ─────────────────────────────────────────────────────

    /// Return which ERA archive a file would be resolved from.
    ///
    /// Checks the override directory first (returns label `"<override>"`),
    /// then walks the ERA stack in reverse load order.
    pub fn provenance(&self, path: &str) -> Option<Provenance> {
        let key = normalise_path(path);

        // Override dir wins.
        if let Some(ref dir) = self.override_dir {
            let fs_path = override_fs_path_scan(dir, &key);
            if fs_path.exists() {
                return Some(Provenance {
                    era_label: "<override>".into(),
                    game_path: key,
                });
            }
        }

        // Last loaded ERA wins.
        for archive in self.archives.iter().rev() {
            if archive.index.contains_key(&key) {
                return Some(Provenance {
                    era_label: archive.label.clone(),
                    game_path: key,
                });
            }
        }
        None
    }

    /// Return which ERA archive a file would be resolved from (with
    /// `.xmb` fallback, matching [`resolve_data`](Self::resolve_data)).
    pub fn provenance_data(&self, path: &str) -> Option<Provenance> {
        if let Some(p) = self.provenance(path) {
            return Some(p);
        }
        // Try appending .xmb (handles "data\objects.xml" → "data\objects.xml.xmb")
        if let Some(p) = self.provenance(&format!("{path}.xmb")) {
            return Some(p);
        }
        // Try replacing .xml with .xmb (handles "art\foo.vis.xml" → "art\foo.vis.xmb")
        if let Some(base) = path.strip_suffix(".xml")
            && let Some(p) = self.provenance(&format!("{base}.xmb"))
        {
            return Some(p);
        }
        None
    }

    /// Like [`provenance`] but **only** checks the ERA archives, ignoring
    /// the override directory.  Used by the write path so that files we've
    /// already written don't shadow the original ERA origin.
    fn provenance_era_only(&self, path: &str) -> Option<Provenance> {
        let key = normalise_path(path);
        for archive in self.archives.iter().rev() {
            if archive.index.contains_key(&key) {
                return Some(Provenance {
                    era_label: archive.label.clone(),
                    game_path: key,
                });
            }
        }
        None
    }

    /// Like [`provenance_data`] but only checks ERA archives (ignores
    /// the override directory).  Used by write helpers to determine the
    /// correct ERA subdirectory without being confused by files we've
    /// already written during the same save.
    fn provenance_data_era_only(&self, path: &str) -> Option<Provenance> {
        if let Some(p) = self.provenance_era_only(path) {
            return Some(p);
        }
        if let Some(p) = self.provenance_era_only(&format!("{path}.xmb")) {
            return Some(p);
        }
        if let Some(base) = path.strip_suffix(".xml")
            && let Some(p) = self.provenance_era_only(&format!("{base}.xmb"))
        {
            return Some(p);
        }
        None
    }

    // ── Override read / write ──────────────────────────────────────────

    /// Write raw bytes to the override directory.
    ///
    /// The file is placed under `{override_dir}/{era_label}/{game_path}`,
    /// where `era_label` is determined by [`provenance`](Self::provenance)
    /// (i.e. which ERA the file originally came from). If the file has no
    /// provenance (new file), it goes under `_new/`.
    ///
    /// Backslash game paths are converted to the OS path separator.
    ///
    /// # Errors
    ///
    /// Returns an error if no override directory is set, or if the
    /// filesystem write fails.
    pub fn write_file(&self, game_path: &str, data: &[u8]) -> Result<PathBuf, String> {
        let dir = self
            .override_dir
            .as_ref()
            .ok_or("no override directory set")?;

        let key = normalise_path(game_path);
        let era_label = self
            .provenance_data_era_only(&key)
            .map(|p| p.era_label)
            .unwrap_or_else(|| "_new".into());

        let fs_path = override_fs_path(dir, &era_label, &key);
        if let Some(parent) = fs_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("failed to create dirs for {}: {e}", fs_path.display()))?;
        }
        std::fs::write(&fs_path, data)
            .map_err(|e| format!("failed to write {}: {e}", fs_path.display()))?;
        Ok(fs_path)
    }

    /// Write an XMB document to the override directory as **binary XMB**.
    ///
    /// Convenience wrapper around [`write_file`](Self::write_file) that
    /// serializes the document with [`xmb::Writer`].  The output path will
    /// have an `.xmb` extension.
    pub fn write_xmb(&self, game_path: &str, doc: &xmb::Document) -> Result<PathBuf, String> {
        let bytes = xmb::Writer::write_native(doc).map_err(|e| format!("XMB write error: {e}"))?;
        let key = normalise_path(game_path);
        let xmb_path = if key.ends_with(".xmb") {
            key
        } else {
            format!("{key}.xmb")
        };
        self.write_file(&xmb_path, &bytes)
    }

    /// Write an XMB document to the override directory as **human-readable XML**.
    ///
    /// The output path will have an `.xml` extension (stripping `.xmb` if
    /// present in the game path).
    pub fn write_xml(&self, game_path: &str, doc: &xmb::Document) -> Result<PathBuf, String> {
        let xml_string = doc.to_xml();
        let key = normalise_path(game_path);
        let xml_path = if key.ends_with(".xmb") {
            key.trim_end_matches(".xmb").to_string()
        } else {
            key
        };
        // Ensure it ends with .xml
        let xml_path = if xml_path.ends_with(".xml") {
            xml_path
        } else {
            format!("{xml_path}.xml")
        };
        self.write_file(&xml_path, xml_string.as_bytes())
    }
}

impl<F: FileProvider> AssetResolver for AssetSource<F> {
    fn resolve(&mut self, path: &str) -> Option<Vec<u8>> {
        // Same fallback as resolve_data — try exact, then append .xmb.
        self.resolve_with_fallback(path, &[".xmb"])
    }

    fn exists(&self, path: &str) -> bool {
        let key = normalise_path(path);
        // Check override dir first.
        if let Some(ref dir) = self.override_dir
            && override_fs_path_scan(dir, &key).exists()
        {
            return true;
        }
        self.archives.iter().any(|a| a.index.contains_key(&key))
    }
}

impl<F: FileProvider> AssetSource<F> {
    /// Resolve an exact path with no extension fallback.
    pub fn resolve_exact(&mut self, path: &str) -> Option<Vec<u8>> {
        let key = normalise_path(path);
        // Check override dir first.
        if let Some(ref dir) = self.override_dir {
            let fs_path = override_fs_path_scan(dir, &key);
            if let Ok(data) = std::fs::read(&fs_path) {
                return Some(data);
            }
        }
        // Last loaded archive wins — iterate in reverse.
        for archive in self.archives.iter_mut().rev() {
            if let Some(&entry_idx) = archive.index.get(&key) {
                return archive.reader.read_entry(entry_idx).ok();
            }
        }
        None
    }
}

// ---------------------------------------------------------------------------
// Override directory helpers
// ---------------------------------------------------------------------------

/// Build the filesystem path for a file in the override directory.
///
/// `{override_dir}/{era_label}/{game_path}` with backslashes converted
/// to the OS separator.
fn override_fs_path(override_dir: &Path, era_label: &str, game_path: &str) -> PathBuf {
    let mut p = override_dir.join(era_label);
    // Convert game path backslashes to OS separators.
    for component in game_path.split('\\') {
        p.push(component);
    }
    p
}

/// Scan all ERA subdirectories in the override dir for a matching file.
///
/// Used by `resolve_exact` and `exists` — we don't know which ERA label
/// the override was written under, so we check all of them.
fn override_fs_path_scan(override_dir: &Path, game_path: &str) -> PathBuf {
    // Build the relative portion once.
    let rel: PathBuf = game_path.split('\\').collect();

    if let Ok(entries) = std::fs::read_dir(override_dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                let candidate = entry.path().join(&rel);
                if candidate.exists() {
                    return candidate;
                }
            }
        }
    }
    // Fallback: return a path that won't exist so callers can just check .exists().
    override_dir.join("_none").join(&rel)
}

// ---------------------------------------------------------------------------
// Concrete FileProvider for std environments
// ---------------------------------------------------------------------------

/// [`FileProvider`] backed by `std::fs::read` — reads entire files into heap.
pub struct StdFileProvider;

impl FileProvider for StdFileProvider {
    type Data = Vec<u8>;

    fn open(&self, path: &str) -> Option<Self::Data> {
        std::fs::read(path).ok()
    }

    fn exists(&self, path: &str) -> bool {
        std::path::Path::new(path).exists()
    }
}

/// Normalise a game path to lowercase with backslash separators.
pub fn normalise_path(path: &str) -> String {
    path.to_lowercase().replace('/', "\\")
}
