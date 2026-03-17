# Focus Engine — System Architecture Specification

> As a synthesis of a Principal Systems Architect and a Cognitive Neuroscientist, I approach the "Focus Engine" not just as a software application, but as a **cognitive prosthetic**.
>
> Human working memory is notoriously volatile. When interrupted, Dr. Gloria Mark's UCI research shows it takes an average of 23 minutes and 15 seconds to return to the original task. Furthermore, Dr. Sophie Leroy's research on **Attention Residue** demonstrates that when we switch tasks without achieving closure, our brain continues to process the incomplete task in the background, drastically reducing our cognitive capacity for the new task.
>
> By utilizing Tauri 2.0's secure, memory-safe Rust foundation, Screenpipe's pervasive local data capture, and MCP (Model Context Protocol) for tool negotiation, we can engineer an OS-level boundary that defends the user's Flow State while satisfying the brain's need for closure.

---

## 1. High-Level System Architecture

The architecture enforces a strict **Zero-Cloud boundary**. The Local LLM acts as an autonomous MCP Client, analyzing context from Screenpipe and negotiating with incoming interruptions via local MCP servers targeting the personal laptop context: iMessage, Discord, and Browser Notifications.

```mermaid
graph TD
    subgraph Frontend[Tauri Frontend: React/TypeScript]
        UI[Focus Dashboard]
        WL[Work/Life Mode Toggle]
        V_CoS[Cost of Switch Visualizer]
        PB_UI[Priority Buffer & Queue]
    end

    subgraph Core[Tauri Backend: Rust Core]
        IPC[Tauri IPC Bridge]
        DB[(Local SQLite + SQLCipher)]
        OS[OS Layer: Global Hotkeys & DND]
        PC[Privacy Config Manager]

        subgraph Engine [Focus Engine Logic]
            CS[Context Snapshot Manager]
            AI[Local LLM / Llama.cpp]
        end
    end

    subgraph Data Layer [Screenpipe Environment]
        SP[Screenpipe API local port]
        OCR[OCR & Window State Logs]
        VCS[VS Code File Path Extractor]
        BCE[Browser Tab URL/Title Extractor]
    end

    subgraph Interruption Layer [MCP Interconnectivity — Local Only]
        MCP_Client[Focus MCP Client]
        MCP_IM[iMessage MCP Server]
        MCP_D[Discord MCP Server]
        MCP_BN[Browser Notification Interceptor]
    end

    %% Frontend to Backend
    UI <-->|Tauri Commands| IPC
    WL <--> IPC
    V_CoS <--> IPC
    PB_UI <--> IPC

    %% Backend internal
    IPC <--> OS
    IPC <--> CS
    IPC <--> PC
    CS <--> DB
    CS -->|Fetch passive context| SP
    SP --> OCR
    SP --> VCS
    SP --> BCE

    %% AI & MCP
    AI <--> MCP_Client
    MCP_Client <-->|Negotiate/Intercept| MCP_IM & MCP_D & MCP_BN
    CS -->|Compute Refocus Time| AI

    %% Styling
    classDef rust fill:#dea584,stroke:#333,stroke-width:2px,color:black;
    classDef react fill:#61dafb,stroke:#333,stroke-width:2px,color:black;
    classDef local fill:#4caf50,stroke:#333,stroke-width:2px,color:black;

    class Core,Engine,OS,CS,DB,PC rust;
    class Frontend,UI,WL,V_CoS,PB_UI react;
    class Data Layer,Interruption Layer,SP,OCR,VCS,BCE,MCP_Client,MCP_IM,MCP_D,MCP_BN local;
```

---

## 2. Neuro-Cognitive Logic of the "Context Snapshot"

To eliminate **Attention Residue**, we must satisfy the **Zeigarnik Effect** (the brain's tendency to fixate on uncompleted tasks). We do this by creating a reliable, externalized "Save State." If the brain trusts the external system, the prefrontal cortex will release the working memory loop.

When the "Freeze Frame" hotkey is triggered, the data must be structured not just technically, but *cognitively*, to minimize the startup friction of returning.

### The Cognitive Data Structure (`snapshot.rs`)

```rust
pub struct ContextSnapshot {
    pub id: String,
    pub timestamp: i64,
    // 1. The Anchoring Intent (What was I doing?)
    // Inferred by Local AI via Screenpipe OCR history over the last 5 minutes.
    pub active_intent: String,

    // 2. Environmental State (Where was I?)
    // Reconstructs the exact pixel/window environment.
    pub open_windows: Vec<WindowState>,
    pub cursor_position: (i32, i32),
    pub visual_context_ocr: String, // Screenpipe snapshot — zeroized on drop

    // 2a. Deep VS Code Context (file path + line number from VS Code window title / OCR)
    pub vscode_context: Option<VsCodeContext>,

    // 2b. Deep Browser Context (URL + title for each Chrome/Arc tab via Screenpipe)
    pub browser_tabs: Vec<BrowserTab>,

    // 3. The Breadcrumb (What is my exact next micro-step?)
    // Crucial for overcoming return-friction. Prompted to the user or AI-generated.
    pub next_immediate_action: String,

    // 4. Cognitive Complexity Score (0.0 - 1.0)
    // Calculated by LLM based on text density, window count, and app types (e.g., IDE + Terminal = High).
    pub cognitive_load_score: f32,

    // 5. Active mode at time of snapshot
    pub mode: WorkLifeMode,
}

pub struct VsCodeContext {
    pub workspace_path: String,
    pub active_file: String,
    pub active_line: Option<u32>,
}

pub struct BrowserTab {
    pub url: String,    // zeroized on drop — may contain credentials in query params
    pub title: String,
    pub is_active: bool,
}

pub enum WorkLifeMode { Work, Personal }
```

**The Restoration Protocol:**
When returning to a snapshot, the app does not just blindly reopen windows. It initiates a **Progressive Re-entry**:
1. Displays the `active_intent` and `next_immediate_action` in large text to prime the neural pathways.
2. Restores windows in the exact Z-order.
3. Places a visual highlight (via OS accessibility APIs) exactly where the cursor was left.

---

## 3. Modular Folder Structure

Tauri 2.0 requires a robust, separation-of-concerns architecture. We isolate OS-level permissions, AI inference, and MCP interactions.

```text
focus-engine/
├── package.json
├── privacy_config.json       # App/window exclusions + incognito settings (user-editable)
├── src/                      # React / TypeScript Frontend
│   ├── components/           # UI: PriorityQueue, SwitchVisualizer, WorkLifeToggle
│   ├── hooks/                # useScreenpipe, useFocusShield, useWorkLifeMode
│   └── lib/                  # Tauri IPC bindings + types
├── src-tauri/                # Tauri 2.0 Rust Core
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/         # Tauri 2.0 Security Capabilities
│   │   └── default.json      # Explicit IPC allowlists
│   └── src/
│       ├── main.rs           # Entry point
│       ├── lib.rs            # App builder and plugin registration
│       ├── ai/               # Local LLM Inference
│       │   ├── mod.rs
│       │   └── engine.rs     # candle/llm-rs bindings for Llama-3-8B
│       ├── commands/         # IPC Handlers mapped to Frontend
│       │   ├── mod.rs
│       │   ├── snapshot.rs   # Trigger Freeze Frame
│       │   ├── os_system.rs  # OS-level Do Not Disturb (DND) toggles
│       │   ├── mode.rs       # Work/Life mode toggle + mode-aware interruption rules
│       │   └── privacy.rs    # Incognito toggle + privacy_config.json management
│       ├── db/               # SQLite + SQLCipher logic
│       │   ├── schema.sql
│       │   └── store.rs
│       ├── mcp/              # Model Context Protocol Bridge
│       │   ├── client.rs     # Autonomously negotiates with MCP Servers
│       │   └── negotiator.rs # Implements the "Priority Buffer" queue logic
│       └── screenpipe/       # Local API Client for Screenpipe
│           ├── client.rs
│           ├── parsers.rs    # Cleans OCR data for the LLM
│           ├── vscode.rs     # Extracts file paths from VS Code window titles/OCR
│           └── browser.rs    # Extracts URLs/titles from Chrome/Arc tab bar OCR
```

**OS-Level DND Toggle (`os_system.rs` snippet):**

```rust
use std::process::Command;

#[tauri::command]
pub fn toggle_do_not_disturb(enable: bool) -> Result<String, String> {
    // Example for macOS using defaults (or applescript/JXA for deeper focus mode integration)
    #[cfg(target_os = "macos")]
    {
        let script = if enable {
            "do shell script \"defaults -currentHost write ~/Library/Preferences/ByHost/com.apple.notificationcenterui doNotDisturb -boolean true\""
        } else {
            "do shell script \"defaults -currentHost write ~/Library/Preferences/ByHost/com.apple.notificationcenterui doNotDisturb -boolean false\""
        };
        Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .map_err(|e| e.to_string())?;

        // Restart Notification Center to apply
        Command::new("killall").arg("NotificationCenter").output().ok();

        Ok(format!("macOS DND set to: {}", enable))
    }
    // Implementations for Windows (Registry/WMI) and Linux (dbus) follow...
}
```

---

## 4. Zero-Cloud Security Manifest

Unlike OpenClaw, which suffered from over-privileged architectures and leaky sandboxes, Focus Engine must operate like a digital SCIF (Sensitive Compartmented Information Facility). Because we are analyzing raw screen OCR (PII, passwords, trade secrets), we implement a **Zero-Trust Local Boundary**.

### 4.1 Network Boundary (Air-Gapping the App)

- The Tauri app is strictly prohibited from making outbound WAN connections.
- In `tauri.conf.json` and the OS firewall profile, the `http` plugin is configured to *only* allow requests to `127.0.0.1` and `localhost` (to communicate with Screenpipe and Local MCP Servers).
- App updates must be signed and fetched via a separate, highly sandboxed updater thread that cannot access the SQLite DB.

### 4.2 Tauri 2.0 Capabilities System

We utilize Tauri V2's capability-based security. Instead of broad `fs` (filesystem) access, we restrict read/write access *exclusively* to the app's isolated `AppLocalData` directory.

```json
// src-tauri/capabilities/default.json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "main-capability",
  "windows": ["main"],
  "permissions":[
    "core:default",
    {
      "identifier": "fs:allow-app-local-data-read-write",
      "allow": [{ "path": "$APPLOCALDATA/**" }]
    },
    "shell:allow-execute" // Strictly scoped to specific OS scripts (osascript)
  ]
}
```

### 4.3 Memory Hardening (Rust `zeroize`)

- Screen OCR data contains highly sensitive PII. When processing the "Cost of Switch" via the Local LLM, we use the `zeroize` crate on Rust structs to ensure that memory is cryptographically zeroed out the exact moment the variable goes out of scope, preventing memory-scraping malware from reading old working memory frames.

### 4.4 MCP Zero-Knowledge Interception (Personal Laptop Targets)

Targets are iMessage, Discord, and Browser Notifications — the primary interruption channels on a personal laptop.

- When the Focus Shield intercepts an iMessage or Discord message via MCP, the incoming payload is parsed locally.
- Focus Engine auto-responds with abstract metadata only: *"User is in a 72%-complexity task. Estimated Refocus Cost: 17 min. Your message has been queued in the Priority Buffer."*
- The exact contents of the user's screen are **never** passed back. Only `cognitive_load_score` leaves the process.

### 4.5 Incognito Mode

A global hotkey (`⌘⇧I`) triggers **Incognito Mode**:

1. All Screenpipe data capture is **immediately suspended** — no new OCR frames, no window state logging.
2. Any in-flight OCR buffers are `zeroize`d in memory.
3. The system tray icon turns **red** as a persistent visual indicator.
4. A `incognito_active: bool` flag is persisted in the app state; it survives restarts.
5. Pressing `⌘⇧I` again or clicking the red tray icon resumes capture.

Incognito mode is intentionally coarse-grained — it is a full kill-switch, not a filter.

### 4.6 Privacy Config (`privacy_config.json`)

A user-editable JSON file at the repo root (bundled into `$APPLOCALDATA` on first run) controls which apps and window titles are **excluded** from data capture.

```json
{
  "excluded_apps": ["1Password", "Keychain Access", "Finder"],
  "excluded_window_title_patterns": ["- Private", "Incognito", "1Password"],
  "redact_urls_matching": ["bank", "paypal", "health"],
  "version": 1
}
```

The Rust `privacy.rs` module reads this config at startup and on `SIGHUP`/file-watch change. Any OCR frame whose source window matches an exclusion rule is discarded before it touches the SQLite DB or the LLM context.

---

## 5. Work/Life Mode Toggle

The **mode toggle** is a first-class UI control that fundamentally changes which incoming signals are treated as interruptions vs. welcome context.

| Signal | Work Mode | Personal Mode |
|---|---|---|
| Discord DM | Interruption (queued) | Allowed through |
| iMessage (personal contact) | Queued unless marked VIP | Allowed through |
| VS Code open | Expected (high-focus) | Interruption signal |
| Chrome — work domain | Expected | Mild interruption |
| Chrome — social/news | Interruption (distraction) | Allowed |

**Implementation:**
- `WorkLifeMode` is an enum stored in the SQLite `app_state` table and in-memory in the Tauri app state.
- The `mode.rs` command module provides `get_mode` and `set_mode` IPC commands.
- `negotiator.rs` consults the current mode before deciding whether an incoming MCP message should be auto-deferred or passed through.
- The MCP auto-response message is mode-aware: in Personal mode, work notifications are deferred; in Work mode, personal notifications are deferred.

---

## 6. Deep Context Extraction

### 6.1 VS Code (`screenpipe/vscode.rs`)

Screenpipe captures window titles. The VS Code window title format is:
`● filename.rs — workspace-name — Visual Studio Code`

The `vscode.rs` parser extracts:
- **Active file path** (relative or absolute, inferred from workspace path)
- **Dirty indicator** (`●` prefix means unsaved changes)
- **Line number** if embedded via a VS Code extension or title override

These fields populate `ContextSnapshot.vscode_context` and are displayed prominently in the Restoration Protocol.

### 6.2 Chrome / Arc Browser (`screenpipe/browser.rs`)

Screenpipe captures the browser's window title, which includes the current tab's page title:
`Focus Engine Architecture - Google Chrome`

For deeper extraction (all open tabs, not just the active one), we query the Screenpipe `/frames` endpoint for recent tab-switch events. The `browser.rs` parser:
- Extracts the page title from window title OCR
- Optionally reads URLs from the address bar via OCR (privacy config may redact these)
- Populates `ContextSnapshot.browser_tabs`

URLs matching `privacy_config.json`'s `redact_urls_matching` patterns are replaced with `[REDACTED]` before any storage or LLM processing.

---

## 7. Milestones

| # | Name | Status | Description |
|---|---|---|---|
| 1 | Hello World | ✅ Done | Tauri 2.0 scaffold, React UI, IPC bridge, capabilities, icons |
| 2 | Screenpipe Integration | ✅ Done | Live OCR frames, privacy filters, app priority, VS Code/browser parsers |
| 3 | SQLite Persistence | ✅ Done | Snapshots and Work/Life mode survive restarts |
| 4 | Local LLM Inference | ✅ Done | `ai_config.json`, smart OCR heuristics, `--features local-llm` GGUF support |
| 5 | MCP Interceptor | 🔲 Next | iMessage + Discord + Browser notification intercept via local MCP |
| 6 | Focus Shield | 🔲 | Global hotkey shield activates DND + queues interruptions via negotiator.rs |
| 7 | Restoration Protocol | 🔲 | Progressive re-entry: display intent/breadcrumb, restore window Z-order |
| 8 | Tray App + Hotkeys | 🔲 | System tray icon, ⌘⇧F freeze, ⌘⇧I incognito global hotkeys wired to OS |
| 9 | Privacy Config UI | 🔲 | In-app editor for privacy_config.json exclusion rules |
| 10 | Open Source Polish | 🔲 | README, install script, model download guide, CI/CD |
