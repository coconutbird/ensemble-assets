# Agent rules

These rules apply to all agents that change this repository. Obey all rules in all changes.

## Language: ASD-STE100 style

Write all new and changed text in a style like ASD-STE100 Simplified Technical English. This
includes doc comments, code comments, Markdown files and commit messages. Follow the writing
rules below. The STE dictionary does not apply: use any clear English word.

- Keep sentences short: about 20 words for an instruction, about 25 words for a description.
- Write one instruction in each sentence.
- Use the imperative form for instructions.
- Use the active voice.
- Prefer simple tenses.
- Give each word one meaning, and use the same term for the same item each time.
- Use the names from the code for code items (for example `AssetSource`, `Database`).
- Use articles ("the", "a") where they make the text clearer.
- Avoid long noun clusters. Break up clusters of more than three words.
- Write short paragraphs with one topic each.
- Use a vertical list for a sequence of steps or for many items.
- Put a warning or a caution before the instruction it applies to.

Write a commit message as `area: summary`, with a lowercase summary. Use the crate or module
name as the area (for example `database`, `pipeline`, `watch`).

## Lints: Clippy pedantic

The workspace denies all warnings, all Clippy `pedantic` lints, `missing_docs` and
`unsafe_code`. `Cargo.toml` sets this in `[workspace.lints]`.

- Keep `[lints] workspace = true` in the `Cargo.toml` file of each crate.
- Do not lower a lint level in a crate or in the code.
- Do not use `#[allow(...)]`. The workspace denies `clippy::allow_attributes`.
- Use `#[expect(lint, reason = "...")]` only if no change to the code can remove the lint.
- Do not add an item to `clippy.toml` to hide a lint.
- Do not add `unsafe` code. The one exception is the file mapping in `StdFileProvider`
  (`crates/pipeline/src/source.rs`). It has `#[expect(unsafe_code)]` and a documented
  invariant.
- Use `zerocopy` to read and write plain data as bytes. Do not use `bytemuck`.
- Make sure that this command completes without warnings:

```sh
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

## File size: 1000 lines

- Keep each source file at a maximum of 1000 lines. This includes integration tests.
- If a change makes a file longer than 1000 lines, split the file into submodules.
- If you change a file that is already longer than 1000 lines, do not make it longer. Split it
  as part of the change.
- Put each submodule in a new file below the directory of its parent module.
- Split by topic. Do not split at a random line.

Clippy also limits each function to 200 lines (`clippy.toml`).

## Layout: Cargo package layout

Follow the Cargo package layout:
<https://doc.rust-lang.org/cargo/guide/project-layout.html>.

- Put the library root in `src/lib.rs`.
- Put the default executable in `src/main.rs`.
- Put other executables in `src/bin/`.
- Put integration tests in `tests/`.
- Put examples in `examples/`.
- Put benchmarks in `benches/`.
- For a test, example, benchmark or executable with many files, make a directory with a `main.rs`
  file and the modules.
- Use snake_case names for modules.
- Put a module in `foo.rs` and its child modules in `foo/`. Do not use `mod.rs` files. The
  workspace denies `clippy::mod_module_files`.
- Put each crate in `crates/<crate-name>/`.

## Layers

The crates of this repository are in four layers:

1. `assets`: `no_std` traits that resolve game assets.
2. `database`: a `no_std` parser that changes XMB documents into typed structs.
3. `pipeline`: ERA loading, scenario loading, asset resolution and the edit and save workflows.
4. Tool crates: `pipeline-cli` and `watch`.

- Make a crate depend only on crates in lower layers.
- Do not do file or network I/O in `assets` or `database`. `pipeline` supplies the bytes.
- Keep `assets` and `database` `no_std`.
- Use the `ensemble-formats` crates to read and write file formats. Put a new format parser or
  writer in `ensemble-formats`, not in this repository.
- Do not make a crate depend on `openensemble`. That repository uses this repository.

## Data rules

- Do not panic on malformed game data. Return an error that tells which file and which value
  are incorrect.
- Keep the shipped game files read-only. Write changes to override files.

## Tests

- Add a regression test for each defect. Make sure that the test fails before the fix.
- Make tests deterministic. Compare floating-point values with a tolerance or by bit pattern.
- If a test needs game files, mark it with `#[ignore = "requires ..."]`. Read the game
  directory from `HW1_GAME_DIR`.
- Run the ignored tests with `cargo test --workspace --all-features -- --ignored`.

## Working copies

- Use one git worktree for each branch when agents work in parallel.
- Use absolute paths for all edits in a worktree. Do not edit the primary checkout from a
  worktree task.

## Checks

Make sure that all these checks pass before you report a change as complete:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo doc --workspace --all-features --no-deps
cargo test --workspace --all-features
```

`prek run --all-files` runs the format and Clippy hooks.
