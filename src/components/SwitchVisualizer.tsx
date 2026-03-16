import type { ContextSnapshot } from "../lib/types";

interface Props {
  snapshot: ContextSnapshot | null;
}

/** Visualizes the cognitive cost of an interruption based on the snapshot's complexity score. */
export function SwitchVisualizer({ snapshot }: Props) {
  if (!snapshot) {
    return (
      <div style={styles.container}>
        <p style={styles.empty}>No snapshot captured yet.</p>
      </div>
    );
  }

  const score = snapshot.cognitive_load_score;
  const refocusMinutes = Math.round(score * 23.25); // Gloria Mark: up to 23m15s
  const barWidth = `${Math.round(score * 100)}%`;

  return (
    <div style={styles.container}>
      <h3 style={styles.title}>Cost of Switch</h3>
      <div style={styles.bar}>
        <div style={{ ...styles.fill, width: barWidth, background: scoreColor(score) }} />
      </div>
      <p style={styles.label}>
        Complexity: <strong>{(score * 100).toFixed(0)}%</strong> — Est. refocus time:{" "}
        <strong style={{ color: scoreColor(score) }}>{refocusMinutes} min</strong>
      </p>
      <p style={styles.intent}>{snapshot.active_intent}</p>
      <p style={styles.next}>
        Next step: <em>{snapshot.next_immediate_action}</em>
      </p>
    </div>
  );
}

function scoreColor(score: number): string {
  if (score < 0.4) return "#4caf50";
  if (score < 0.7) return "#ff9800";
  return "#ef5350";
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    background: "var(--bg-surface)",
    border: "1px solid var(--border)",
    borderRadius: 8,
    padding: "20px 24px",
  },
  title: {
    fontSize: 13,
    fontWeight: 600,
    color: "var(--text-secondary)",
    textTransform: "uppercase",
    letterSpacing: "0.08em",
    marginBottom: 12,
  },
  bar: {
    height: 6,
    background: "var(--bg-elevated)",
    borderRadius: 3,
    overflow: "hidden",
    marginBottom: 10,
  },
  fill: {
    height: "100%",
    borderRadius: 3,
    transition: "width 0.4s ease",
  },
  label: { fontSize: 13, color: "var(--text-secondary)", marginBottom: 12 },
  intent: { fontSize: 15, fontWeight: 500, marginBottom: 6 },
  next: { fontSize: 13, color: "var(--text-secondary)" },
  empty: { color: "var(--text-secondary)", fontStyle: "italic", fontSize: 13 },
};
