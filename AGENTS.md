# AGENTS.md — project notes for AI assistants working on OpenTermius

## Build & run
- Desktop: `cd desktop && npm install && npm run tauri dev`
- Rust not yet installed in dev env; install via rustup first.
- Workspace root is the repo root; `cargo build` from root builds core + desktop.

## Verification
- `cargo check --workspace` after core changes.
- `cargo test -p opentermius-core` once tests are added.
- Tauri build: `cd desktop && npm run tauri build`.

## Conventions
- All SSH/crypto logic lives in `core/`, never in shells.
- Private key material must be wrapped to `Zeroize` on drop.
- Host key mismatches never auto-accept; surface to user.
- No secrets in logs; no plaintext keys on disk; vault is the only persisted form.
- Pin dependency versions (no floating `latest` / `*`).

## Git conventions
- NEVER mention, attribute, or add a co-author trailer for Devin or any AI
  agent in commit messages. No "Generated with Devin", no
  "Co-Authored-By: Devin", no AI attribution of any kind. Commits should look
  like they were written by the human author only.
- Commit messages: concise, focus on "why" not "what".

## Architecture
See `docs/ARCHITECTURE.md`. One Rust core, Tauri desktop shell now, mobile
via FFI later.
