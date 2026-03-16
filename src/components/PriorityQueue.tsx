/** Priority Buffer queue — stub for incoming intercepted interruptions. */
export function PriorityQueue() {
  // Placeholder: real data will come from the MCP negotiator via Tauri IPC
  const items: { id: string; source: string; preview: string; priority: number }[] = [];

  return (
    <div style={styles.container}>
      <h3 style={styles.title}>Priority Buffer</h3>
      {items.length === 0 ? (
        <p style={styles.empty}>No queued interruptions. Your focus is uncontested.</p>
      ) : (
        <ul style={styles.list}>
          {items.map((item) => (
            <li key={item.id} style={styles.item}>
              <span style={styles.source}>{item.source}</span>
              <span style={styles.preview}>{item.preview}</span>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
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
  empty: { color: "var(--text-secondary)", fontStyle: "italic", fontSize: 13 },
  list: { listStyle: "none" },
  item: {
    display: "flex",
    gap: 12,
    padding: "8px 0",
    borderBottom: "1px solid var(--border)",
  },
  source: { fontWeight: 600, minWidth: 60, fontSize: 13 },
  preview: { color: "var(--text-secondary)", fontSize: 13 },
};
