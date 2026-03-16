import { useState, useCallback } from "react";
import { freezeFrame } from "../lib/commands";
import type { ContextSnapshot } from "../lib/types";

export function useFreezeFrame() {
  const [snapshot, setSnapshot] = useState<ContextSnapshot | null>(null);
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
