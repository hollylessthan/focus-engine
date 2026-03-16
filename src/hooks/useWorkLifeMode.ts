import { useState, useCallback, useEffect } from "react";
import { getMode, setMode } from "../lib/commands";
import type { WorkLifeMode } from "../lib/types";

export function useWorkLifeMode() {
  const [mode, setModeState] = useState<WorkLifeMode>("work");
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    getMode()
      .then(setModeState)
      .catch(() => {}); // silently default to "work" if command fails
  }, []);

  const changeMode = useCallback(async (next: WorkLifeMode) => {
    setLoading(true);
    try {
      await setMode(next);
      setModeState(next);
    } finally {
      setLoading(false);
    }
  }, []);

  return { mode, loading, changeMode };
}
