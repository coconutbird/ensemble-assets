//! HW1 localized string table parser.
//!
//! Parses `data\stringtable-{lang}.xml` (XMB) from `locale.era`.
//!
//! ## Format
//!
//! ```xml
//! <StringTable>
//!   <Language name="English">
//!     <String _locID="100" category="Code" subtitle="false">CONFIRM</String>
//!     <String _locID="101" category="Code" subtitle="false">CANCEL</String>
//!     ...
//!   </Language>
//! </StringTable>
//! ```
//!
//! The `_locID` maps to the `display_name_id`, `rollover_text_id`, etc.
//! fields in the database types.

use std::collections::HashMap;

use crate::source::AssetSource;
use database::node_ext::NodeExt;

/// Supported language codes, matching the `data\stringtable-{code}.xml`
/// filenames found in `locale.era`.
///
/// In the Definitive Edition only `locale.era` ships (per-language
/// update ERAs like `locale_en-us_update.era` are dead code from the
/// original retail build — see `BLocaleManager__resolveLocaleERAs`
/// in IDA).
pub const LANGUAGE_CODES: &[&str] = &[
    "en", "de", "es", "fr", "it", "ja", "ko", "zh", "cs", "hu", "pl", "ru",
];

/// A loaded string table: maps `_locID` (i32) → localized text.
#[derive(Debug, Clone, Default)]
pub struct StringTable {
    /// The language code (e.g. `"en"`, `"de"`).
    pub language_code: String,
    /// The language name from the XML (e.g. `"English"`, `"German"`).
    pub language_name: String,
    /// Strings keyed by `_locID`.
    pub strings: HashMap<i32, StringEntry>,
}

/// A single localized string entry.
#[derive(Debug, Clone)]
pub struct StringEntry {
    /// The localized text (default / gamepad version).
    pub text: String,
    /// Alternate text for mouse+keyboard input. When present, the engine
    /// uses this instead of `text` on PC (`_mouseKeyboard` attribute in
    /// the XML). `None` means no override — use `text` for all inputs.
    pub mouse_keyboard: Option<String>,
    /// Category tag (e.g. `"Code"`, `"Objects"`, `"Powers"`).
    pub category: String,
    /// Whether this is a subtitle string.
    pub subtitle: bool,
}

impl StringTable {
    /// Load a string table for a specific language code from the asset source.
    ///
    /// Returns `None` if the file is not found.
    pub fn load(lang: &str, src: &mut AssetSource<impl assets::FileProvider>) -> Option<Self> {
        let path = format!("data\\stringtable-{lang}.xml");
        let doc = src.read_xmb(&path)?;
        let root = doc.root()?;

        if root.name != "StringTable" {
            return None;
        }

        let lang_node = root.children.first()?;
        let language_name = lang_node.attr_str("name").unwrap_or_default();

        let mut strings = HashMap::new();

        for entry in &lang_node.children {
            if entry.name != "String" {
                continue;
            }
            let Some(loc_id) = entry.attr_i32("_locID") else {
                continue;
            };
            let text = entry.text_string();
            let mouse_keyboard = entry.attr_str("_mouseKeyboard");
            let category = entry.attr_str("category").unwrap_or_default();
            let subtitle = entry.attr_bool("subtitle").unwrap_or(false);

            strings.insert(
                loc_id,
                StringEntry {
                    text,
                    mouse_keyboard,
                    category,
                    subtitle,
                },
            );
        }

        Some(StringTable {
            language_code: lang.to_string(),
            language_name,
            strings,
        })
    }

    /// Look up a string by its `_locID` (default / gamepad text).
    pub fn get(&self, loc_id: i32) -> Option<&str> {
        self.strings.get(&loc_id).map(|e| e.text.as_str())
    }

    /// Look up a string by its `_locID`, preferring the mouse+keyboard
    /// override when available. This is what the engine uses on PC.
    pub fn get_pc(&self, loc_id: i32) -> Option<&str> {
        self.strings
            .get(&loc_id)
            .map(|e| e.mouse_keyboard.as_deref().unwrap_or(&e.text))
    }

    /// Look up a string entry by its `_locID`.
    pub fn get_entry(&self, loc_id: i32) -> Option<&StringEntry> {
        self.strings.get(&loc_id)
    }

    /// Number of strings in this table.
    pub fn len(&self) -> usize {
        self.strings.len()
    }

    /// Whether this table is empty.
    pub fn is_empty(&self) -> bool {
        self.strings.is_empty()
    }
}

/// Load the default (English) string table.
pub fn load_default(src: &mut AssetSource<impl assets::FileProvider>) -> Option<StringTable> {
    StringTable::load("en", src)
}
