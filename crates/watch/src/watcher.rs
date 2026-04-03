//! [`WorldWatcher`] — filesystem watcher + incremental reload + validation.

use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

use notify_debouncer_mini::{DebouncedEventKind, Debouncer, new_debouncer};

use pipeline::hw1::{AssetKind, DiagnosticReport, TableId, World, validate_world};
use pipeline::source::{AssetSource, StdFileProvider};

/// Events emitted by the [`WorldWatcher`] when the override directory changes.
#[derive(Debug)]
pub enum WorldEvent {
    /// An asset was successfully reloaded from disk.
    AssetReloaded(AssetKind),
    /// Legacy alias for database-table reloads.
    TableReloaded(TableId),
    /// Cross-reference diagnostics were re-computed after a reload.
    DiagnosticsUpdated(DiagnosticReport),
    /// A file changed but could not be mapped to a known asset type.
    UnknownFile(PathBuf),
    /// An error occurred during reload or validation.
    Error(String),
}

/// Watches the override directory and incrementally reloads the [`World`].
///
/// # Usage
///
/// ```ignore
/// let mut watcher = WorldWatcher::new(world, src, override_dir)?;
/// watcher.start()?;
///
/// // Engine hot-reload loop:
/// for event in watcher.poll() {
///     match event {
///         WorldEvent::TableReloaded(t) => println!("reloaded {t:?}"),
///         WorldEvent::DiagnosticsUpdated(r) => r.print_summary(),
///         _ => {}
///     }
/// }
///
/// // Access the world:
/// let hp = watcher.world().database.objects[0].hitpoints;
/// ```
pub struct WorldWatcher {
    world: World,
    src: AssetSource<StdFileProvider>,
    override_dir: PathBuf,
    event_rx: mpsc::Receiver<WorldEvent>,
    event_tx: mpsc::Sender<WorldEvent>,
    /// Internal notify debouncer — `None` until [`start`](Self::start) is called.
    _debouncer: Option<Debouncer<notify::RecommendedWatcher>>,
    /// Internal channel for raw notify events.
    notify_rx:
        Option<mpsc::Receiver<Result<Vec<notify_debouncer_mini::DebouncedEvent>, notify::Error>>>,
}

impl WorldWatcher {
    /// Create a new watcher (not yet watching).
    ///
    /// Call [`start`](Self::start) to begin filesystem monitoring.
    pub fn new(
        world: World,
        src: AssetSource<StdFileProvider>,
        override_dir: impl Into<PathBuf>,
    ) -> Self {
        let (event_tx, event_rx) = mpsc::channel();
        Self {
            world,
            src,
            override_dir: override_dir.into(),
            event_rx,
            event_tx,
            _debouncer: None,
            notify_rx: None,
        }
    }

    /// Start watching the override directory for changes.
    pub fn start(&mut self) -> Result<(), notify::Error> {
        let (tx, rx) = mpsc::channel();
        let mut debouncer = new_debouncer(Duration::from_millis(200), tx)?;

        // Watch recursively.
        debouncer
            .watcher()
            .watch(&self.override_dir, notify::RecursiveMode::Recursive)?;

        self._debouncer = Some(debouncer);
        self.notify_rx = Some(rx);
        Ok(())
    }

    /// Immutable access to the world.
    pub fn world(&self) -> &World {
        &self.world
    }

    /// Mutable access to the world.
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    /// Mutable access to the asset source.
    pub fn source_mut(&mut self) -> &mut AssetSource<StdFileProvider> {
        &mut self.src
    }

    /// Save dirty tables to the override directory (convenience wrapper).
    ///
    /// Equivalent to `self.world.save(&self.src)` but avoids borrow conflicts.
    pub fn save(&mut self) -> Result<Vec<std::path::PathBuf>, String> {
        self.world.save(&self.src)
    }

    /// Non-blocking: drain all pending notify events, process them,
    /// and return any resulting [`WorldEvent`]s.
    ///
    /// Call this each frame (engine) or on a timer (LSP).
    pub fn poll(&mut self) -> Vec<WorldEvent> {
        self.drain_notify_events();
        let mut events = Vec::new();
        while let Ok(ev) = self.event_rx.try_recv() {
            events.push(ev);
        }
        events
    }

    /// Blocking: wait for the next [`WorldEvent`].
    ///
    /// Returns `None` if the channel is closed.
    pub fn recv(&mut self) -> Option<WorldEvent> {
        self.drain_notify_events();
        // First check if we already have events queued.
        if let Ok(ev) = self.event_rx.try_recv() {
            return Some(ev);
        }
        // Wait for a notify event, process it, then return the result.
        if let Some(rx) = self.notify_rx.take() {
            let result = rx.recv();
            self.notify_rx = Some(rx);
            if let Ok(result) = result {
                self.process_notify_result(result);
                return self.event_rx.try_recv().ok();
            }
        }
        None
    }

    /// Manually trigger a reload of a specific database table and re-validate.
    ///
    /// For per-file assets (visuals, models, etc.) use [`reload_asset`](Self::reload_asset).
    pub fn reload(&mut self, table: TableId) -> Vec<WorldEvent> {
        self.reload_asset(AssetKind::DatabaseTable(table))
    }

    /// Manually trigger a reload of any asset kind and re-validate.
    ///
    /// This is the universal reload entry point. Works for database
    /// tables, per-object XML files, and binary assets.
    pub fn reload_asset(&mut self, kind: AssetKind) -> Vec<WorldEvent> {
        let mut events = Vec::new();
        match self.world.reload_asset(&kind, &mut self.src) {
            Ok(true) => {
                events.push(WorldEvent::AssetReloaded(kind));
                let report = validate_world(&self.world);
                events.push(WorldEvent::DiagnosticsUpdated(report));
            }
            Ok(false) => {
                events.push(WorldEvent::Error(format!(
                    "asset {kind:?}: source not found"
                )));
            }
            Err(e) => {
                events.push(WorldEvent::Error(format!(
                    "asset {kind:?}: reload failed: {e}"
                )));
            }
        }
        events
    }

    /// Stop watching (drops the debouncer).
    pub fn stop(&mut self) {
        self._debouncer = None;
        self.notify_rx = None;
    }

    // -- internal --

    fn drain_notify_events(&mut self) {
        // Take the receiver out temporarily to avoid borrow conflicts.
        let Some(rx) = self.notify_rx.take() else {
            return;
        };
        while let Ok(result) = rx.try_recv() {
            self.process_notify_result(result);
        }
        self.notify_rx = Some(rx);
    }

    fn process_notify_result(
        &mut self,
        result: Result<Vec<notify_debouncer_mini::DebouncedEvent>, notify::Error>,
    ) {
        match result {
            Ok(events) => {
                // Deduplicate: collect unique AssetKinds from the batch.
                let mut seen = std::collections::HashSet::new();
                let mut unknown = Vec::new();

                for ev in &events {
                    if !matches!(
                        ev.kind,
                        DebouncedEventKind::Any | DebouncedEventKind::AnyContinuous
                    ) {
                        continue;
                    }

                    match crate::asset_for_override_path(&self.override_dir, &ev.path) {
                        Some(kind) => {
                            seen.insert(kind);
                        }
                        None => {
                            unknown.push(ev.path.clone());
                        }
                    }
                }

                // Reload each affected asset once.
                let mut reloaded_any = false;
                for kind in seen {
                    match self.world.reload_asset(&kind, &mut self.src) {
                        Ok(true) => {
                            let _ = self.event_tx.send(WorldEvent::AssetReloaded(kind));
                            reloaded_any = true;
                        }
                        Ok(false) => {
                            let _ = self.event_tx.send(WorldEvent::Error(format!(
                                "asset {kind:?}: source not found after change"
                            )));
                        }
                        Err(e) => {
                            let _ = self.event_tx.send(WorldEvent::Error(format!(
                                "asset {kind:?}: reload failed: {e}"
                            )));
                        }
                    }
                }

                // Re-validate once after all reloads.
                if reloaded_any {
                    let report = validate_world(&self.world);
                    let _ = self.event_tx.send(WorldEvent::DiagnosticsUpdated(report));
                }

                for path in unknown {
                    let _ = self.event_tx.send(WorldEvent::UnknownFile(path));
                }
            }
            Err(e) => {
                let _ = self
                    .event_tx
                    .send(WorldEvent::Error(format!("notify error: {e}")));
            }
        }
    }
}
