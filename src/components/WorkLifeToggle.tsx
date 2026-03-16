import type { WorkLifeMode } from "../lib/types";

interface Props {
  mode: WorkLifeMode;
  onChange: (mode: WorkLifeMode) => void;
}

/** Work/Life mode toggle — changes which signals are treated as interruptions. */
export function WorkLifeToggle({ mode, onChange }: Props) {
  return (
    <div style={styles.container}>
      <span style={styles.label}>Mode</span>
      <div style={styles.toggle}>
        <button
          style={{
            ...styles.option,
            ...(mode === "work" ? styles.activeWork : styles.inactive),
          }}
          onClick={() => onChange("work")}
        >
          Work
        </button>
        <button
          style={{
            ...styles.option,
            ...(mode === "personal" ? styles.activePersonal : styles.inactive),
          }}
          onClick={() => onChange("personal")}
        >
          Personal
        </button>
      </div>
      <span style={styles.hint}>
        {mode === "work"
          ? "Discord & iMessage are queued as interruptions"
          : "VS Code activity is flagged; personal channels pass through"}
      </span>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    display: "flex",
    alignItems: "center",
    gap: 12,
    flexWrap: "wrap",
  },
  label: {
    fontSize: 12,
    fontWeight: 600,
    color: "var(--text-secondary)",
    textTransform: "uppercase",
    letterSpacing: "0.08em",
  },
  toggle: {
    display: "flex",
    background: "var(--bg-elevated)",
    borderRadius: 6,
    padding: 2,
    border: "1px solid var(--border)",
  },
  option: {
    padding: "5px 14px",
    borderRadius: 4,
    fontSize: 12,
    fontWeight: 600,
    transition: "background 0.15s, color 0.15s",
    background: "transparent",
  },
  activeWork: {
    background: "var(--accent)",
    color: "#1a0f0a",
  },
  activePersonal: {
    background: "#4caf50",
    color: "#0a1a0a",
  },
  inactive: {
    color: "var(--text-secondary)",
  },
  hint: {
    fontSize: 11,
    color: "var(--text-secondary)",
    fontStyle: "italic",
  },
};
