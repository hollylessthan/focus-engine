import { useState, useCallback } from "react";
import { SwitchVisualizer } from "./components/SwitchVisualizer";
import { PriorityQueue } from "./components/PriorityQueue";
import { WorkLifeToggle } from "./components/WorkLifeToggle";
import { useFreezeFrame } from "./hooks/useFreezeFrame";
import { useFocusShield } from "./hooks/useFocusShield";
import { useScreenpipe } from "./hooks/useScreenpipe";
import { useWorkLifeMode } from "./hooks/useWorkLifeMode";
import { toggleIncognito, getIncognitoStatus } from "./lib/commands";
import { useEffect } from "react";

export default function App() {
  const { snapshot, loading: freezeLoading, error: snapshotError, trigger: triggerFreezeFrame } = useFreezeFrame();
  const { status: shieldStatus, activate, deactivate } = useFocusShield();
  const screenpipe = useScreenpipe();
  const { mode, changeMode } = useWorkLifeMode();

  const [incognito, setIncognito] = useState(false);

  useEffect(() => {
    getIncognitoStatus()
      .then(setIncognito)
      .catch(() => {});
  }, []);

  const handleIncognitoToggle = useCallback(async () => {
    try {
      const next = await toggleIncognito();
      setIncognito(next);
    } catch {
      /* stub — ignore */
    }
  }, []);

  return (
    <div style={styles.root}>
      {/* Title bar */}
      <header style={{ ...styles.header, borderBottomColor: incognito ? "var(--red)" : "var(--border)" }}>
        <div style={styles.headerLeft}>
          <span style={{ ...styles.logo, color: incognito ? "var(--red)" : "var(--accent)" }}>◉</span>
          <span style={styles.appName}>Focus Engine</span>
          <span style={styles.version}>v0.1.0</span>
          {incognito && (
            <span style={styles.incognitoBadge}>INCOGNITO — Capture Paused</span>
          )}
        </div>
        <div style={styles.headerRight}>
          <WorkLifeToggle mode={mode} onChange={changeMode} />
          <div style={styles.divider} />
          <span
            style={{ ...styles.pipeDot, background: screenpipe.connected ? "var(--green)" : "#555" }}
          />
          <span style={styles.pipeLabel}>
            {screenpipe.connected ? "Screenpipe" : "Screenpipe offline"}
          </span>
        </div>
      </header>

      {/* Main content */}
      <main style={styles.main}>
        {/* Shield + controls */}
        <section style={styles.shieldSection}>
          <div
            style={{
              ...styles.shieldBadge,
              background: shieldStatus === "active" ? "var(--accent-dim)" : "var(--bg-elevated)",
              borderColor: shieldStatus === "active" ? "var(--accent)" : "var(--border)",
            }}
          >
            <span
              style={{
                ...styles.shieldIndicator,
                background: shieldStatus === "active" ? "var(--accent)" : "#555",
              }}
            />
            <span style={styles.shieldLabel}>
              Focus Shield:{" "}
              <strong style={{ color: shieldStatus === "active" ? "var(--accent)" : "var(--text-secondary)" }}>
                {shieldStatus === "active" ? "ACTIVE" : "Inactive"}
              </strong>
            </span>
          </div>

          <div style={styles.controls}>
            <button
              style={{ ...styles.btn, ...styles.btnPrimary, opacity: freezeLoading ? 0.6 : 1 }}
              onClick={triggerFreezeFrame}
              disabled={freezeLoading}
              title="Save your current cognitive context — ⌘⇧F"
            >
              {freezeLoading ? "Capturing…" : "Freeze Frame  ⌘⇧F"}
            </button>

            {shieldStatus === "inactive" ? (
              <button style={{ ...styles.btn, ...styles.btnSecondary }} onClick={activate}>
                Activate Shield
              </button>
            ) : (
              <button style={{ ...styles.btn, ...styles.btnDanger }} onClick={deactivate}>
                Deactivate Shield
              </button>
            )}

            <button
              style={{
                ...styles.btn,
                ...(incognito ? styles.btnDanger : styles.btnSecondary),
              }}
              onClick={handleIncognitoToggle}
              title="Kill all data capture — ⌘⇧I"
            >
              {incognito ? "● Incognito ON  ⌘⇧I" : "Incognito  ⌘⇧I"}
            </button>
          </div>

          {snapshotError && <p style={styles.errorMsg}>Error: {snapshotError}</p>}
        </section>

        {/* Cost-of-Switch visualizer */}
        <SwitchVisualizer snapshot={snapshot} />

        {/* Priority buffer */}
        <PriorityQueue />
      </main>

      {/* Status bar */}
      <footer style={styles.footer}>
        <span>Zero-Cloud  •  All data local  •  Mode: <strong>{mode}</strong></span>
        <span>SQLite + SQLCipher  •  Local LLM  •  {incognito ? "🔴 Incognito" : "Capturing"}</span>
      </footer>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  root: { display: "flex", flexDirection: "column", height: "100vh", background: "var(--bg-primary)" },
  header: {
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
    padding: "0 16px",
    height: 48,
    background: "var(--bg-surface)",
    borderBottom: "1px solid var(--border)",
    flexShrink: 0,
    gap: 16,
  },
  headerLeft: { display: "flex", alignItems: "center", gap: 10, flexShrink: 0 },
  logo: { fontSize: 18, transition: "color 0.2s" },
  appName: { fontWeight: 700, fontSize: 14, letterSpacing: "0.02em" },
  version: { fontSize: 11, color: "var(--text-secondary)", paddingTop: 2 },
  incognitoBadge: {
    fontSize: 10,
    fontWeight: 700,
    letterSpacing: "0.1em",
    color: "var(--red)",
    background: "rgba(239, 83, 80, 0.15)",
    border: "1px solid var(--red)",
    borderRadius: 4,
    padding: "2px 6px",
  },
  headerRight: { display: "flex", alignItems: "center", gap: 10 },
  divider: { width: 1, height: 20, background: "var(--border)" },
  pipeDot: { width: 7, height: 7, borderRadius: "50%", flexShrink: 0 },
  pipeLabel: { fontSize: 11, color: "var(--text-secondary)" },
  main: {
    flex: 1,
    padding: 24,
    display: "flex",
    flexDirection: "column",
    gap: 16,
    overflowY: "auto",
  },
  shieldSection: { display: "flex", flexDirection: "column", gap: 12 },
  shieldBadge: {
    display: "inline-flex",
    alignItems: "center",
    gap: 10,
    padding: "10px 16px",
    borderRadius: 8,
    border: "1px solid",
    alignSelf: "flex-start",
  },
  shieldIndicator: { width: 8, height: 8, borderRadius: "50%", flexShrink: 0 },
  shieldLabel: { fontSize: 14 },
  controls: { display: "flex", gap: 10, flexWrap: "wrap" },
  btn: {
    padding: "9px 18px",
    borderRadius: 6,
    fontSize: 13,
    fontWeight: 600,
    transition: "opacity 0.15s",
  },
  btnPrimary: { background: "var(--accent)", color: "#1a0f0a" },
  btnSecondary: {
    background: "var(--bg-elevated)",
    color: "var(--text-primary)",
    border: "1px solid var(--border)",
  },
  btnDanger: {
    background: "rgba(239, 83, 80, 0.2)",
    color: "var(--red)",
    border: "1px solid var(--red)",
  },
  errorMsg: { fontSize: 12, color: "var(--red)" },
  footer: {
    display: "flex",
    justifyContent: "space-between",
    padding: "0 20px",
    height: 28,
    background: "var(--bg-surface)",
    borderTop: "1px solid var(--border)",
    alignItems: "center",
    fontSize: 11,
    color: "var(--text-secondary)",
    flexShrink: 0,
  },
};
