# Focus Engine — Project Standards

This file defines the coding standards, architectural rules, and conventions for the Focus Engine project. All contributors (human and AI) must follow these guidelines.

---

## Stack

| Layer | Technology |
|---|---|
| Frontend | React 18 + TypeScript + Vite |
| Backend | Rust (stable) + Tauri 2.0 |
| Database | SQLite + SQLCipher via `rusqlite` |
| AI Inference | Local only — `candle` or `llama-cpp-rs` (Llama-3-8B) |
| IPC | Tauri Commands (`#[tauri::command]`) |
| Screenpipe | Local HTTP API on `127.0.0.1` |
| MCP | Local MCP client/server only |

---

## Zero-Cloud Security Rules (Non-Negotiable)

1. **No outbound WAN connections.** All HTTP requests are restricted to `127.0.0.1` and `localhost`. No external API calls, ever. No telemetry.
2. **No API keys.** AI inference runs entirely local. If you're importing an OpenAI/Anthropic/etc. client, you've made an error.
3. **No screen data leaves the machine.** OCR text from Screenpipe is processed in-process only. Only the abstract `cognitive_load_score` (a float) may be shared with MCP servers.
4. **`zeroize` all PII-adjacent structs.** Any Rust struct that holds OCR text, window titles, or user content must derive or implement `zeroize::Zeroize`. This ensures memory is wiped when the struct is dropped.
5. **Tauri capabilities are minimal.** Only grant `$APPLOCALDATA/**` for filesystem access. No broad `fs` permissions. `shell:allow-execute` is scoped to specific OS scripts only.

---

## Rust Conventions

### Module Layout (`src-tauri/src/`)

```
main.rs            — #[cfg_attr(not(debug_assertions), windows_subsystem = "windows")] fn main()
lib.rs             — tauri::Builder setup; registers ALL commands and plugins here
ai/                — Local LLM inference engine (no network)
commands/          — ALL #[tauri::command] handlers live here, nowhere else
  snapshot.rs      — freeze_frame, list_snapshots
  os_system.rs     — toggle_do_not_disturb
  mode.rs          — get_mode, set_mode (Work/Life toggle)
  privacy.rs       — toggle_incognito, get_privacy_config, update_privacy_config
db/                — SQLite/SQLCipher connection, migrations, queries
mcp/               — Local MCP client and priority buffer negotiator (iMessage, Discord, Browser)
screenpipe/        — HTTP client for local Screenpipe API + OCR parsers
  vscode.rs        — Extracts file path/line from VS Code window titles
  browser.rs       — Extracts URL/title from Chrome/Arc tab bar OCR
```

### Command Registration

All Tauri commands **must** be registered in `lib.rs`. Do not call `.invoke_handler()` anywhere else.

```rust
// lib.rs — the only place commands are registered
.invoke_handler(tauri::generate_handler![
    commands::snapshot::freeze_frame,
    commands::os_system::toggle_do_not_disturb,
])
```

### Error Handling

- Tauri commands return `Result<T, String>` — convert errors with `.map_err(|e| e.to_string())`.
- Never `unwrap()` or `expect()` in command handlers. Propagate errors to the frontend.
- Use `thiserror` for domain error types within modules.

### Memory Safety for PII

```rust
use zeroize::Zeroize;

#[derive(Zeroize)]
#[zeroize(drop)]
pub struct OcrFrame {
    pub text: String,
    pub timestamp: i64,
}
```

Apply `#[zeroize(drop)]` so sensitive data is cleared automatically on drop.

### Formatting & Linting

```bash
cargo fmt                          # must pass
cargo clippy -- -D warnings        # must produce zero warnings
```

Configure in `src-tauri/.cargo/config.toml` if needed.

---

## TypeScript/React Conventions

### Frontend Structure (`src/`)

```
components/    — Presentational components only; no direct Tauri calls
hooks/         — All Tauri invoke calls live in custom hooks (e.g., useFreezeFrame)
lib/           — Typed wrappers around `@tauri-apps/api/core` invoke
App.tsx        — Root component; composes hooks and components
```

### Tauri IPC Pattern

Never call `invoke` directly in a component. Use the typed wrappers in `src/lib/commands.ts`:

```typescript
// src/lib/commands.ts
import { invoke } from "@tauri-apps/api/core";

export async function freezeFrame(): Promise<ContextSnapshot> {
  return invoke<ContextSnapshot>("freeze_frame");
}
```

### Type Definitions

Mirror Rust structs as TypeScript interfaces in `src/lib/types.ts`. Keep them in sync manually — there is no codegen step yet.

### Formatting & Linting

```bash
npm run lint    # eslint
npm run format  # prettier
```

Use `strict` TypeScript mode. No `any` types.

---

## Database

- All schema changes go in `src-tauri/src/db/schema.sql` as incremental `CREATE TABLE IF NOT EXISTS` statements.
- Run migrations on app startup in `db/store.rs`.
- SQLCipher key is derived from the OS keychain — never hardcoded.
- No ORM. Use raw `rusqlite` queries.

---

## Testing

- **Rust:** Unit tests in each module file (`#[cfg(test)]` blocks). Integration tests for all Tauri commands in `src-tauri/tests/`.
- **TypeScript:** Vitest for unit tests on hooks and lib functions.
- Run before every commit:

```bash
cargo test --manifest-path src-tauri/Cargo.toml
npm test
```

---

## Git Conventions

- Branch: `feature/<name>`, `fix/<name>`, `chore/<name>`
- Commits: conventional commits format (`feat:`, `fix:`, `chore:`, `docs:`)
- Never commit secrets, model weights, or `.env` files
- `src-tauri/gen/` is gitignored (Tauri build artifacts)

---

## Work/Life Mode

- `WorkLifeMode` (`Work` | `Personal`) is stored in SQLite `app_state` table AND in Tauri's managed `AppState` struct.
- Mode changes propagate synchronously: `set_mode` command → update `AppState` → persist to DB → `negotiator.rs` reads current mode on every MCP intercept.
- The mode toggle affects `negotiator.rs`'s allow/defer decision. See SPEC.md Section 5 for the full rule table.
- Never hard-code app names as "always work" or "always personal" — the rule table lives in `privacy_config.json`.

## Incognito Mode

- Triggered by `⌘⇧I` global hotkey → calls `toggle_incognito` Tauri command.
- When active: Screenpipe polling **stops immediately**; any buffered OCR is `zeroize`d; tray icon turns red.
- Incognito state is stored in `AppState.incognito_active: bool` (in-memory) only — it does **not** persist across restarts (restarts resume capture).
- All Tauri commands that would normally read Screenpipe data must check `AppState.incognito_active` and return early if true.

## Privacy Config (`privacy_config.json`)

- Lives at `$APPLOCALDATA/focus-engine/privacy_config.json` (copied from repo root on first launch).
- Loaded by `privacy.rs` on startup; re-read on file change (file watcher or explicit reload command).
- Any OCR frame whose source window title matches an `excluded_window_title_patterns` pattern is dropped in `screenpipe/parsers.rs` **before** hitting the DB or LLM.
- URLs in `browser_tabs` matching `redact_urls_matching` patterns are replaced with `[REDACTED]` in `screenpipe/browser.rs` before any struct is serialized.

## MCP Targets (Personal Laptop)

The three local MCP integration targets are:
1. **iMessage** — intercept via `mcp/client.rs` connecting to a local iMessage MCP server
2. **Discord** — intercept via Discord's local RPC port (`127.0.0.1:6463`) or a local MCP server wrapper
3. **Browser Notifications** — intercept via the browser's notification permission system or a local proxy

All three share the same zero-knowledge response pattern: only `cognitive_load_score` and `refocus_minutes` are included in auto-responses. The `negotiator.rs` respects the current `WorkLifeMode` when deciding to defer or allow each source.

## What NOT to Do

- Do not add cloud SDKs, analytics, crash reporters, or feature flags.
- Do not add `tokio::spawn` threads that outlive the Tauri app lifecycle without a shutdown signal.
- Do not store user data outside `$APPLOCALDATA`.
- Do not make the frontend aware of OS-specific logic — that belongs in Rust commands.
- Do not create helper abstractions for one-off operations. Three similar lines of code beat a premature abstraction.
