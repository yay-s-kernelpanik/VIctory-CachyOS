# Contributing to VIctory (Linux port)

Thank you for helping port VIctory to Linux/CachyOS. Please read this
before opening a pull request.

## Project scope

- **Native Linux only.** No Wine or Proton. The host backend is Vulkan
  (`crates/victory-gpu/src/host_vulkan.rs`).
- **GPL-3.0-or-later.** All contributions are licensed under GPL-3.0-or-later.
- **No game assets or executables.** Never commit, distribute, or embed
  `game_scarlett_master.exe`, shaders, textures, audio, or any other
  title data. Contributors must own a legal copy and provide their own
  files locally. See `docs/asset-policy.md`.

## Getting started

```bash
# Build
cargo build --release -p victory

# Tests
cargo test --workspace
cargo test -p xtask policy

# Vulkan smoke
./target/release/victory gpu-check
./target/release/victory test-vulkan
```

Requirements: Rust toolchain (see `rust-toolchain.toml`), Vulkan drivers,
and optionally `spirv-val`/`spirv-as` (SPIRV-Tools) for shader validation.

## Branching and pull requests

- Branch from `feat/vulkan-minimo` (or `main` when merged).
- Use descriptive branch names: `feat/vulkan-...`, `fix/...`, `docs/...`.
- Open a pull request; do not push directly to `main`.
- Keep changes focused: one logical change per PR.

## Commit messages

Follow the existing style:

```
feat(vulkan): dispatch a compute shader in gpu-check
fix(translate): avoid duplicate RPF notes
docs(linux): document SPIR-V validation
test(translate): cover spirv-val acceptance
```

- Prefix with `feat`/`fix`/`docs`/`test`/`refactor` and a scope in
  parentheses.
- Write one or two concise sentences on **why**, not just what.
- End with the attribution footer unless the repository has asked
  otherwise:

```
🤖 Generated with Codebuff
Co-Authored-By: Codebuff <noreply@codebuff.com>
```

## Code style

- Run `cargo fmt` on the files you touched (avoid `cargo fmt --all`,
  which reformats unrelated files).
- Keep `cargo clippy` clean for changed crates.
- Match existing conventions: `#[cfg(windows)]` for platform-specific
  code, `#[cfg(target_os = "linux")]` for Linux-only code.
- Do not add new dependencies without checking they are already used
  in the workspace.

## Testing

- Every change must have a verification command and its output.
- Add or update tests for new logic; run `cargo test --workspace`
  before submitting.
- Do not skip, weaken, or delete tests to make CI pass.
- Report any test that could not run as a limitation, not a pass.

## What not to do

- Do not port the loader (`victory_loader::load`) or HLE modules unless
  you are taking on that architectural work explicitly.
- Do not modify `emit/` or `host_vulkan.rs` in glue/mechanical tasks.
- Do not commit game data, executables, or extracted shader packs.
- Do not run `cargo fmt --all` and commit unrelated reformatting.
- Do not push to `main` or force-push shared branches.

## Platform notes

- Linux port targets CachyOS; test on a real Vulkan device when possible.
- `translate --pipelines` is still pending on Linux.
- RPF9 archives require external extraction (`rage-tools-vi`) or the
  graceful stub path.

## Reporting issues

When reporting a failure, include:

- Exact command and its full output.
- OS/kernel, GPU, driver, Vulkan version (`victory gpu-check`).
- Whether a legal game executable was used (do not share it).

## See also

- `docs/ROADMAP.md` — file-by-file audit and todo list.
- `docs/ROADMAP-LINUX.md` — Linux milestones.
- `docs/AUDITORIA-REAL.md` — real audit.
- `README.md` — project overview.