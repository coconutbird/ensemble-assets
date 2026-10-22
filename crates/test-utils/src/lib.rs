//! Shared test infrastructure for ensemble-assets.
//!
//! Add `test-utils` as a `[dev-dependency]`. Use [`require_game_dir`] in each test that needs
//! the installed game, and mark that test with `#[ignore = "requires HW1_GAME_DIR"]`.

use std::path::Path;
use std::sync::Once;

/// Load the `.env` file from the workspace root one time.
///
/// A variable that is already set in the environment keeps its value.
fn load_dotenv() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let env_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.env");
        // The `.env` file is optional. The environment can supply the variables.
        let _ = dotenvy::from_path(env_path);
    });
}

/// Read a game directory from an environment variable.
///
/// This function loads the workspace `.env` file first. It returns a `String` because the
/// `pipeline` loaders take the game directory as `&str`.
///
/// Use this function only in tests that have `#[ignore = "requires ..."]`. These tests run only
/// when you request them, so a missing game directory is a configuration error.
///
/// # Panics
///
/// Panics if the variable is unset, empty or does not name a directory.
#[must_use]
pub fn require_game_dir(env_name: &str) -> String {
    load_dotenv();
    let dir = std::env::var(env_name)
        .ok()
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| {
            panic!(
                "{env_name} is not set: set it in the environment or in the workspace `.env` \
                 file to the game directory"
            )
        });
    assert!(
        Path::new(&dir).is_dir(),
        "{env_name}={dir} is not a directory"
    );
    dir
}
