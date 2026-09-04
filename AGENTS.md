# AGENTS.md — project notes for AI assistants working on OpenTermius

## Build & run
- Desktop: `cd desktop && npm install && npm run tauri dev`
- Rust not yet installed in dev env; install via rustup first.
- Workspace root is the repo root; `cargo build` from root builds core + desktop.

## Verification
- `cargo check --workspace` after core changes.
- `cargo test -p opentermius-core` once tests are added.
- Tauri build: `cd desktop && npm run tauri build`.
- Frontend typecheck: `cd desktop && npx vue-tsc --noEmit`.

## Building distributable versions
- Local build (current platform only): `cd desktop && npm run tauri build`
  - macOS: produces `.dmg` + `.app` in `desktop/src-tauri/target/release/bundle/`
  - Linux: produces `.deb`, `.AppImage`, `.rpm`
  - Windows: produces `.msi` + `.exe` (NSIS)
- Cross-platform releases via GitHub Actions:
  - Push a tag `v0.1.0` → triggers `.github/workflows/release.yml`
  - Builds on macOS (arm64 + x86_64), Linux, Windows runners
  - Signs update packages with `TAURI_SIGNING_PRIVATE_KEY` secret
  - Publishes artifacts + `latest.json` to GitHub Releases (draft)

## Auto-update
- Uses `tauri-plugin-updater` (configured in `tauri.conf.json` under `plugins.updater`)
- Endpoint: `https://github.com/CodeAny-inc/OpenTermius/releases/latest/download/latest.json`
- To switch to a custom CDN later: change the `endpoints` array in `tauri.conf.json`
- App checks for updates on startup (release builds only, silent)
- Frontend shows `UpdateBanner` when an update is available
- User clicks "Download & Restart" → downloads, verifies signature, installs, restarts
- Signing key: private key is a GitHub secret (`TAURI_SIGNING_PRIVATE_KEY`),
  public key is embedded in `tauri.conf.json`

## Required GitHub secrets for releases
- `TAURI_SIGNING_PRIVATE_KEY` — the base64 private key (from `tauri signer generate`)
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` — empty string if key has no password

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
