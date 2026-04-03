//! Dirty tracking for the edit/save workflow.

use std::cell::Cell;
use std::ops::{Deref, DerefMut};

/// Identifies a data table in a [`World`](super::World).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum TableId {
    Objects,
    Squads,
    Techs,
    Abilities,
    Powers,
    Civs,
    Leaders,
    WeaponTypes,
    DamageTypes,
    GameData,
    Scenario,
    Visuals,
    Tactics,
    Physics,
}

impl TableId {
    pub const COUNT: usize = 14;

    pub const ALL: [TableId; Self::COUNT] = [
        Self::Objects,
        Self::Squads,
        Self::Techs,
        Self::Abilities,
        Self::Powers,
        Self::Civs,
        Self::Leaders,
        Self::WeaponTypes,
        Self::DamageTypes,
        Self::GameData,
        Self::Scenario,
        Self::Visuals,
        Self::Tactics,
        Self::Physics,
    ];

    fn index(self) -> usize {
        self as u8 as usize
    }
}

/// Tracks which tables have been modified.
pub struct DirtySet {
    flags: [Cell<bool>; TableId::COUNT],
}

impl Default for DirtySet {
    fn default() -> Self {
        Self {
            flags: std::array::from_fn(|_| Cell::new(false)),
        }
    }
}

impl DirtySet {
    pub fn is_dirty(&self, table: TableId) -> bool {
        self.flags[table.index()].get()
    }

    pub fn is_any_dirty(&self) -> bool {
        self.flags.iter().any(Cell::get)
    }

    pub fn dirty_tables(&self) -> Vec<TableId> {
        TableId::ALL
            .iter()
            .copied()
            .filter(|&t| self.is_dirty(t))
            .collect()
    }

    pub fn clear(&self) {
        for flag in &self.flags {
            flag.set(false);
        }
    }

    pub(crate) fn flag(&self, table: TableId) -> &Cell<bool> {
        &self.flags[table.index()]
    }
}

/// RAII guard that marks a table dirty when dropped.
///
/// Returned by `World::*_mut()` accessors. Dereferences to `&mut T`
/// so callers can mutate the data naturally. The dirty flag is set on
/// drop, not on each write — this avoids overhead and keeps the guard
/// cheap.
pub struct DirtyGuard<'a, T> {
    data: &'a mut T,
    flag: &'a Cell<bool>,
}

impl<'a, T> DirtyGuard<'a, T> {
    pub(crate) fn new(data: &'a mut T, flag: &'a Cell<bool>) -> Self {
        Self { data, flag }
    }
}

impl<T> Deref for DirtyGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.data
    }
}

impl<T> DerefMut for DirtyGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.data
    }
}

impl<T> Drop for DirtyGuard<'_, T> {
    fn drop(&mut self) {
        self.flag.set(true);
    }
}
