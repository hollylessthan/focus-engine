import { useState, useCallback, useEffect } from "react";
import { freezeFrame, listSnapshots } from "../lib/commands";
import type { ContextSnapshot } from "../lib/types";

export function useFreezeFrame() {
  const [snapshot, setSnapshot] = useState<ContextSnapshot | null>(null);

  // Restore the most recent snapshot from DB on startup so the visualizer
  // isn't blank after a restart.
  useEffect(() => {
    listSnapshots().then((snapshots) => {
      if (snapshots.length > 0) setSnapshot(snapshots[0]);
    }).catch(() => {});
  }, []);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const trigger = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await freezeFrame();
      setSnapshot(result);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  return { snapshot, loading, error, trigger };
}
