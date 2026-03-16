import { useState, useCallback } from "react";
import { toggleDoNotDisturb } from "../lib/commands";
import type { ShieldStatus } from "../lib/types";

export function useFocusShield() {
  const [status, setStatus] = useState<ShieldStatus>("inactive");
  const [error, setError] = useState<string | null>(null);

  const activate = useCallback(async () => {
    try {
      await toggleDoNotDisturb(true);
      setStatus("active");
    } catch (err) {
      setError(String(err));
    }
  }, []);

  const deactivate = useCallback(async () => {
    try {
      await toggleDoNotDisturb(false);
      setStatus("inactive");
    } catch (err) {
      setError(String(err));
    }
  }, []);

  return { status, error, activate, deactivate };
}
