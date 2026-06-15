//! ERA-backed asset resolution with optional filesystem override layer.
//!
//! [`AssetSource`] is the single entry point for reading (and writing)
//! game assets. It layers multiple ERA archives on top of each other
//! following the engine's priority rule: **last loaded ERA wins**
//! (matching `BFileManager::resolveFile` in the engine binary).
//!
//! An optional **override directory** sits above the entire ERA stack.
//! When set, any file written via [`AssetSource::write_file`] or
//! [`AssetSource::write_xmb`] lands in
//! `{override_dir}/{era_label}/{game_path}`, and subsequent reads will
//! find that override before consulting the ERAs.
//!
//! # ERA load order
//!
//! The engine loads archives in a fixed sequence (see [`hw1::loader`]):
//!
//! ```text
//! locale.era          ← lowest priority
//! root.era / root_update.era
//! shader.era
//! miniloader.era / pregameUI.era
//! ingameUI.era
//! scenarioshared.era
//! dlc01..10.era
//! {scenario}.era      ← highest priority (loaded on map start)
//! ```
//!
//! When two ERAs contain the same file (e.g. `data\objects.xml.xmb`),
//! the one loaded **later** wins — exactly like the real engine.
//!
//! # Example
//!
//! ```no_run
//! use pipeline::source::{AssetSource, StdFileProvider};
//!
//! let mut src = AssetSource::with_provider(StdFileProvider);
//! src.add_era("/path/to/game/root.era").expect("open ERA");
//!
//! // Read a file — tries exact match, then appends .xmb as fallback.
//! if let Some(data) = src.resolve_data("data\\objects.xml") {
//!     println!("objects.xml is {} bytes", data.len());
//! }
//!
//! // Find which ERA a file comes from.
//! if let Some(prov) = src.provenance_data("data\\objects.xml") {
//!     println!("resolved from {}", prov.era_label);
//! }
//! ```
//!
//! [`hw1::loader`]: crate::hw1::loader

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use assets::{AssetResolver, FileProvider};

/// A loaded ERA archive with a pre-built filename → entry index map.
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

/// How a folder layer combines with the layers below it when the stack is
/// flattened.
///
/// An ERA and a loose `ModData` folder are the same thing — a set of
/// game-path → bytes entries — so a mod is just an unpacked archive layered on
/// top of the base game. The rule decides what a layer contributes:
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LoadRule {
    /// Last-writer-wins: this layer's version of a path overrides lower layers'
    /// (the engine's ERA behaviour). The default.
    #[default]
    Replace,
    /// Only contribute paths that no lower layer already provides; never
    /// override an existing file.
    Additive,
}

/// A loose directory layered onto the source — an unpacked archive.
///
/// Indexed exactly like an ERA: normalised game-path → the file on disk.
struct FolderLayer {
    /// Layer label for diagnostics (the folder's name).
    label: String,
    /// Normalised game path → absolute file path on disk.
    index: HashMap<String, PathBuf>,
    /// How this layer composes during a flatten.
    rule: LoadRule,
}

/// One path written by more than one folder layer during a flatten — the
/// later layer's version won.
#[derive(Debug, Clone)]
pub struct Overwrite {
    /// Normalised game path that was provided by multiple layers.
    pub game_path: String,
    /// Label of the layer whose version was overridden.
    pub previous: String,
    /// Label of the layer whose version was written.
    pub winner: String,
}

/// Outcome of flattening the folder layers to a directory.
#[derive(Debug, Default)]
pub struct FlattenReport {
    /// Files written to the output directory.
    pub files_written: Vec<PathBuf>,
    /// Paths that more than one layer provided (later wins).
    pub overwrites: Vec<Overwrite>,
    /// Files skipped because they were byte-identical to the base game.
    pub skipped_vanilla: usize,
}

/// Unified asset source backed by one or more ERA archives, with an
/// optional filesystem override layer for read/write support.
///
/// Resolution order (first match wins):
///
/// 1. **Override directory** — files previously saved via [`write_file`](Self::write_file)
/// 2. **Folder layers** — loose directories ([`add_folder`](Self::add_folder)),
///    later-added wins (reverse order)
/// 3. **ERA stack** — last loaded archive wins (reverse load order)
///
/// The generic parameter `F` is the [`FileProvider`] used to open ERA
/// files from disk. Use [`StdFileProvider`] for normal filesystem access.
pub struct AssetSource<F: FileProvider> {
    provider: F,
    archives: Vec<LoadedArchive<F::Data>>,
    /// Loose-folder layers stacked above the ERAs (e.g. mod `ModData`).
    folders: Vec<FolderLayer>,
    /// Optional override directory for read/write support.
    override_dir: Option<PathBuf>,
    /// The source directory this asset source was built from (if any).
    source_dir: Option<String>,
}

impl<F: FileProvider> AssetSource<F> {
    /// Create an empty asset source with the given file provider.
    pub fn with_provider(provider: F) -> Self {
        Self {
            provider,
            archives: Vec::new(),
            folders: Vec::new(),
            override_dir: None,
            source_dir: None,
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

    /// Add a loose directory as a layer above the ERA stack.
    ///
    /// A mod's `ModData` folder is just an unpacked ERA — a set of
    /// game-path → file entries — so indexing it as a layer lets the same
    /// resolution and flatten logic treat packed archives and loose folders
    /// uniformly. Folder layers sit above every ERA (a mod overrides the base
    /// game), and a later `add_folder` wins over an earlier one.
    ///
    /// Files are indexed by their path relative to `dir`, normalised the same
    /// way as ERA entries (lowercase, backslash separators). Returns the number
    /// of files indexed.
    pub fn add_folder(&mut self, dir: &str, rule: LoadRule) -> Result<usize, String> {
        let root = PathBuf::from(dir);
        if !root.is_dir() {
            return Err(format!("not a directory: {dir}"));
        }

        let mut index = HashMap::new();
        let mut stack = vec![root.clone()];
        while let Some(d) = stack.pop() {
            let entries =
                std::fs::read_dir(&d).map_err(|e| format!("read_dir {}: {e}", d.display()))?;
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                    continue;
                }
                if let Ok(rel) = path.strip_prefix(&root) {
                    let key = normalise_path(&rel.to_string_lossy());
                    index.insert(key, path);
                }
            }
        }

        let count = index.len();
        let label = root
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| dir.to_string());
        self.folders.push(FolderLayer {
            label,
            index,
            rule,
        });
        Ok(count)
    }

    /// Number of folder layers currently loaded.
    pub fn folder_count(&self) -> usize {
        self.folders.len()
    }

    /// Read `path` from **every** folder layer that provides it, in load order
    /// (lowest priority first). Each folder is probed for the unpacked form and
    /// then the packed `.xmb` form, mirroring [`resolve_data`](Self::resolve_data).
    ///
    /// Unlike [`resolve_data`], which returns only the top layer's version, this
    /// surfaces each layer's version — the input a schema-aware merge needs to
    /// combine the same table across several mods.
    pub fn read_each_folder_data(&self, path: &str) -> Vec<(String, Vec<u8>)> {
        let key = normalise_path(path);
        let key_xmb = format!("{key}.xmb");
        let mut out = Vec::new();
        for layer in &self.folders {
            if let Some(fs_path) = layer.index.get(&key).or_else(|| layer.index.get(&key_xmb))
                && let Ok(bytes) = std::fs::read(fs_path)
            {
                out.push((layer.label.clone(), bytes));
            }
        }
        out
    }

    /// Flatten the folder layers into `out_dir`, writing each provided file at
    /// its game path. This is the write-out half of "load layers, resolve,
    /// write the result" — the merged mod is just the flattened stack.
    ///
    /// - `skip` holds normalised paths handled elsewhere (e.g. database tables
    ///   resolved by a schema-aware merge); both the unpacked and packed forms
    ///   of each skipped path are excluded.
    /// - With `only_changed`, a file byte-identical to the base game (ERA) is
    ///   not written, so the output contains only what actually differs from
    ///   vanilla.
    /// - [`LoadRule::Additive`] layers contribute a path only if no lower layer
    ///   already provides it; [`LoadRule::Replace`] layers always win.
    ///
    /// Paths provided by more than one layer are reported as [`Overwrite`]s.
    pub fn flatten_folders_to_dir(
        &mut self,
        out_dir: &Path,
        skip: &std::collections::HashSet<String>,
        only_changed: bool,
    ) -> Result<FlattenReport, String> {
        // Plan with an immutable borrow: for each path, the ordered layer
        // indices that contribute it, honouring each layer's rule.
        let mut providers: std::collections::BTreeMap<String, Vec<usize>> =
            std::collections::BTreeMap::new();
        for (i, layer) in self.folders.iter().enumerate() {
            for key in layer.index.keys() {
                if skip.contains(key) {
                    continue;
                }
                let slot = providers.entry(key.clone()).or_default();
                // Additive only contributes when nothing below it (or at the
                // same level) has already claimed the path.
                if layer.rule == LoadRule::Additive && !slot.is_empty() {
                    continue;
                }
                slot.push(i);
            }
        }

        struct PlanItem {
            key: String,
            src: PathBuf,
            previous: Option<String>,
            winner: String,
        }
        let plan: Vec<PlanItem> = providers
            .into_iter()
            .filter_map(|(key, idxs)| {
                let &win = idxs.last()?;
                let src = self.folders[win].index.get(&key)?.clone();
                let previous = (idxs.len() > 1)
                    .then(|| self.folders[idxs[idxs.len() - 2]].label.clone());
                Some(PlanItem {
                    key,
                    src,
                    previous,
                    winner: self.folders[win].label.clone(),
                })
            })
            .collect();

        let mut report = FlattenReport::default();
        for item in plan {
            let bytes = std::fs::read(&item.src)
                .map_err(|e| format!("read {}: {e}", item.src.display()))?;

            if only_changed
                && self.resolve_vanilla_data(&item.key).as_deref() == Some(bytes.as_slice())
            {
                report.skipped_vanilla += 1;
                continue;
            }

            let dest = join_game_path(out_dir, &item.key);
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("create dirs {}: {e}", parent.display()))?;
            }
            std::fs::write(&dest, &bytes).map_err(|e| format!("write {}: {e}", dest.display()))?;
            report.files_written.push(dest);

            if let Some(previous) = item.previous {
                report.overwrites.push(Overwrite {
                    game_path: item.key,
                    previous,
                    winner: item.winner,
                });
            }
        }
        Ok(report)
    }

    /// Remove the last ERA archive from the stack.
    ///
    /// Returns the label of the removed archive, or `None` if the stack
    /// was empty. This is used by [`World::swap_scenario`](crate::hw1::World::swap_scenario)
    /// to pop a scenario ERA before pushing a different one.
    pub fn pop_era(&mut self) -> Option<String> {
        self.archives.pop().map(|a| a.label)
    }

    /// Number of ERA archives currently loaded.
    pub fn era_count(&self) -> usize {
        self.archives.len()
    }

    /// Set the source directory this asset source was built from.
    pub fn set_source_dir(&mut self, dir: impl Into<String>) {
        self.source_dir = Some(dir.into());
    }

    /// Return the source directory, if set.
    pub fn source_dir(&self) -> Option<&str> {
        self.source_dir.as_deref()
    }

    /// Add an ERA from the source directory by filename.
    ///
    /// Requires [`set_source_dir`](Self::set_source_dir) to have been called.
    /// Returns `Ok(entry_count)` on success, or `Err` if source_dir is unset
    /// or the ERA cannot be loaded.
    pub fn add_era_by_name(&mut self, era_name: &str) -> Result<usize, String> {
        let dir = self
            .source_dir
            .as_deref()
            .ok_or_else(|| "source_dir not set on AssetSource".to_string())?;
        let path = format!("{dir}/{era_name}");
        if !std::path::Path::new(&path).exists() {
            return Err(format!("ERA not found: {path}"));
        }
        self.add_era(&path)
    }

    /// Find the ERA filename for a scenario by name.
    ///
    /// Accepts an exact ERA filename (`"PHXscn01.era"`), a map name
    /// (`"blood_gulch"`), or a full SCN path. Scans the source directory
    /// for a matching `.era` file.
    ///
    /// Requires [`set_source_dir`](Self::set_source_dir) to have been called.
    pub fn find_scenario_era(&self, scenario: &str) -> Option<String> {
        let dir = self.source_dir.as_deref()?;
        crate::hw1::loader::find_scenario_era(dir, scenario)
    }

    /// Resolve a file, trying `suffixes` as fallback extensions.
    ///
    /// Override files are checked for **all** variants first, ensuring
    /// that a saved `.xmb` override wins over an ERA's `.xml` original.
    pub fn resolve_with_fallback(&mut self, path: &str, suffixes: &[&str]) -> Option<Vec<u8>> {
        // 1. Try all variants in the override directory first.
        if let Some(data) = self.resolve_override(path) {
            return Some(data);
        }
        for suffix in suffixes {
            let fallback = format!("{path}{suffix}");
            if let Some(data) = self.resolve_override(&fallback) {
                return Some(data);
            }
        }
        // 2. Then the folder layers (later-added wins).
        if let Some(data) = self.resolve_folder(path) {
            return Some(data);
        }
        for suffix in suffixes {
            let fallback = format!("{path}{suffix}");
            if let Some(data) = self.resolve_folder(&fallback) {
                return Some(data);
            }
        }
        // 3. Fall back to ERA archives.
        if let Some(data) = self.resolve_era(path) {
            return Some(data);
        }
        for suffix in suffixes {
            let fallback = format!("{path}{suffix}");
            if let Some(data) = self.resolve_era(&fallback) {
                return Some(data);
            }
        }
        None
    }

    /// Resolve a path's bytes from the **base game only** (ERA archives),
    /// ignoring folder layers and overrides — the "vanilla" view used to decide
    /// whether a flattened file actually differs from the base game. Tries the
    /// packed `.xmb` variant as a fallback, like [`resolve_data`](Self::resolve_data).
    pub fn resolve_vanilla_data(&mut self, path: &str) -> Option<Vec<u8>> {
        if let Some(data) = self.resolve_era(path) {
            return Some(data);
        }
        self.resolve_era(&format!("{path}.xmb"))
    }

    /// Resolve a data file by its real path (e.g. `data\objects.xml`,
    /// `art\foo.vis`), trying the compiled `.xmb` variant as fallback.
    pub fn resolve_data(&mut self, path: &str) -> Option<Vec<u8>> {
        self.resolve_with_fallback(path, &[".xmb"])
    }

    /// Read and parse an XMB document, with `.xmb` fallback.
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

    /// Like [`provenance`] but only checks ERA archives (ignores overrides).
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

    /// Like [`provenance_data`] but only checks ERA archives (ignores overrides).
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

    /// Write raw bytes to `{override_dir}/{era_label}/{game_path}`.
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

    /// Write an XMB document as human-readable XML to the override directory.
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
        if self.folders.iter().any(|f| f.index.contains_key(&key)) {
            return true;
        }
        self.archives.iter().any(|a| a.index.contains_key(&key))
    }
}

impl<F: FileProvider> AssetSource<F> {
    /// Resolve an exact path with no extension fallback.
    pub fn resolve_exact(&mut self, path: &str) -> Option<Vec<u8>> {
        if let Some(data) = self.resolve_override(path) {
            return Some(data);
        }
        self.resolve_era(path)
    }

    /// Resolve an exact path from the override directory only.
    fn resolve_override(&self, path: &str) -> Option<Vec<u8>> {
        let key = normalise_path(path);
        let dir = self.override_dir.as_ref()?;
        let fs_path = override_fs_path_scan(dir, &key);
        std::fs::read(&fs_path).ok()
    }

    /// Resolve an exact path from ERA archives only (last-loaded wins).
    fn resolve_era(&mut self, path: &str) -> Option<Vec<u8>> {
        let key = normalise_path(path);
        for archive in self.archives.iter_mut().rev() {
            if let Some(&entry_idx) = archive.index.get(&key) {
                return archive.reader.read_entry(entry_idx).ok();
            }
        }
        None
    }

    /// Resolve an exact path from the folder layers only (later-added wins).
    fn resolve_folder(&self, path: &str) -> Option<Vec<u8>> {
        let key = normalise_path(path);
        for layer in self.folders.iter().rev() {
            if let Some(fs_path) = layer.index.get(&key) {
                return std::fs::read(fs_path).ok();
            }
        }
        None
    }
}

// Override directory helpers

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

/// Build an output filesystem path from a normalised game path, converting
/// backslash separators to the OS separator.
fn join_game_path(out_dir: &Path, game_path: &str) -> PathBuf {
    let mut p = out_dir.to_path_buf();
    for component in game_path.split('\\') {
        p.push(component);
    }
    p
}

/// Scan all ERA subdirectories in the override dir for a matching file.
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

// Concrete FileProvider for std environments

/// [`FileProvider`] backed by `std::fs::read` — reads entire files into heap.
impl AssetSource<StdFileProvider> {
    /// Load a scenario ERA by name (map name, ERA filename, or SCN path).
    ///
    /// Finds the matching ERA in the source directory and pushes it onto the
    /// archive stack. Returns `true` if the ERA was found and loaded.
    ///
    /// Requires [`set_source_dir`](Self::set_source_dir) to have been called.
    pub fn load_scenario(&mut self, scenario: &str) -> bool {
        let dir = match self.source_dir.as_deref() {
            Some(d) => d.to_string(),
            None => return false,
        };
        if let Some(era_name) = crate::hw1::loader::find_scenario_era(&dir, scenario) {
            crate::hw1::loader::load_scenario_era(self, &dir, &era_name)
        } else {
            eprintln!("  WARN  could not find scenario ERA for '{scenario}'");
            false
        }
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::sync::atomic::{AtomicU32, Ordering};

    fn tmp(label: &str) -> PathBuf {
        static N: AtomicU32 = AtomicU32::new(0);
        let p = std::env::temp_dir().join(format!(
            "asset_src_{label}_{}_{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&p);
        p
    }

    /// Folders index like archives, resolve top-wins, surface every layer's
    /// version, and flatten out (skipping excluded paths, reporting overwrites).
    #[test]
    fn folder_layers_resolve_and_flatten() {
        let root = tmp("folders");
        let (a, b, out) = (root.join("a"), root.join("b"), root.join("out"));
        std::fs::create_dir_all(a.join("data")).unwrap();
        std::fs::create_dir_all(a.join("art")).unwrap();
        std::fs::create_dir_all(b.join("art")).unwrap();
        std::fs::write(a.join("art/icon.ddx"), b"a-icon").unwrap();
        std::fs::write(a.join("data/objects.xml"), b"a-obj").unwrap();
        std::fs::write(b.join("art/icon.ddx"), b"b-icon").unwrap(); // overrides A
        std::fs::write(b.join("art/banner.ddx"), b"b-banner").unwrap();

        let mut src = AssetSource::with_provider(StdFileProvider);
        assert_eq!(src.add_folder(&a.to_string_lossy(), LoadRule::Replace).unwrap(), 2);
        assert_eq!(src.add_folder(&b.to_string_lossy(), LoadRule::Replace).unwrap(), 2);
        assert_eq!(src.folder_count(), 2);

        // Later layer wins on resolve.
        assert_eq!(src.resolve_data("art\\icon.ddx").as_deref(), Some(&b"b-icon"[..]));
        // read_each surfaces both versions, in load order.
        let each = src.read_each_folder_data("art\\icon.ddx");
        assert_eq!(each.len(), 2);
        assert_eq!(each[0].1, b"a-icon");
        assert_eq!(each[1].1, b"b-icon");

        // Flatten, skipping the table path. only_changed=false (no base ERAs).
        let mut skip = HashSet::new();
        skip.insert(normalise_path("data\\objects.xml"));
        let report = src.flatten_folders_to_dir(&out, &skip, false).unwrap();

        assert_eq!(std::fs::read(out.join("art/icon.ddx")).unwrap(), b"b-icon"); // B won
        assert_eq!(std::fs::read(out.join("art/banner.ddx")).unwrap(), b"b-banner");
        assert!(!out.join("data/objects.xml").exists()); // skipped table path
        assert_eq!(report.overwrites.len(), 1);
        assert_eq!(report.overwrites[0].game_path, "art\\icon.ddx");
        assert_eq!(report.overwrites[0].winner, "b");

        let _ = std::fs::remove_dir_all(&root);
    }

    /// An additive layer contributes only paths no lower layer already provides.
    #[test]
    fn additive_layer_does_not_override() {
        let root = tmp("additive");
        let (a, b, out) = (root.join("a"), root.join("b"), root.join("out"));
        std::fs::create_dir_all(&a).unwrap();
        std::fs::create_dir_all(&b).unwrap();
        std::fs::write(a.join("shared.txt"), b"from-a").unwrap();
        std::fs::write(b.join("shared.txt"), b"from-b").unwrap();
        std::fs::write(b.join("extra.txt"), b"from-b").unwrap();

        let mut src = AssetSource::with_provider(StdFileProvider);
        src.add_folder(&a.to_string_lossy(), LoadRule::Replace).unwrap();
        src.add_folder(&b.to_string_lossy(), LoadRule::Additive).unwrap();

        let report = src.flatten_folders_to_dir(&out, &HashSet::new(), false).unwrap();

        // B is additive: it adds `extra` but does not override A's `shared`.
        assert_eq!(std::fs::read(out.join("shared.txt")).unwrap(), b"from-a");
        assert_eq!(std::fs::read(out.join("extra.txt")).unwrap(), b"from-b");
        assert!(report.overwrites.is_empty());

        let _ = std::fs::remove_dir_all(&root);
    }
}
