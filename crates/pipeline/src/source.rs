//! ERA-backed asset resolution using a [`FileProvider`] for I/O.
//!
//! [`AssetSource`] owns the ERA resolution logic (load order, index lookups,
//! decompression) while delegating raw file reads to a generic
//! [`FileProvider`].  The CLI uses [`StdFileProvider`] (backed by
//! `std::fs::read`); an engine could swap in memory-mapped I/O.
//!
//! The "last loaded wins" priority rule matches the engine's
//! `BFileManager::resolveFile` (confirmed in IDA at 0x140807090).

use std::collections::HashMap;

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

/// Unified asset source backed by one or more ERA archives.
///
/// Generic over a [`FileProvider`] that supplies the raw ERA file bytes.
/// Resolution follows the engine rule: **highest load order (last loaded) wins**.
pub struct AssetSource<F: FileProvider> {
    provider: F,
    archives: Vec<LoadedArchive<F::Data>>,
}

impl<F: FileProvider> AssetSource<F> {
    /// Create an empty asset source with the given file provider.
    pub fn with_provider(provider: F) -> Self {
        Self {
            provider,
            archives: Vec::new(),
        }
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
}

impl<F: FileProvider> AssetResolver for AssetSource<F> {
    fn resolve(&mut self, path: &str) -> Option<Vec<u8>> {
        // Same fallback as resolve_data — try exact, then append .xmb.
        self.resolve_with_fallback(path, &[".xmb"])
    }

    fn exists(&self, path: &str) -> bool {
        let key = normalise_path(path);
        self.archives.iter().any(|a| a.index.contains_key(&key))
    }
}

impl<F: FileProvider> AssetSource<F> {
    /// Resolve an exact path with no extension fallback.
    pub fn resolve_exact(&mut self, path: &str) -> Option<Vec<u8>> {
        let key = normalise_path(path);
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
