import { useState, useEffect } from "react";
import type { ScreenpipeStatus } from "../lib/types";

const SCREENPIPE_URL = "http://127.0.0.1:3030";

export function useScreenpipe() {
  const [status, setStatus] = useState<ScreenpipeStatus>({
    connected: false,
    last_frame_at: null,
  });

  useEffect(() => {
    let cancelled = false;

    async function probe() {
      try {
        const res = await fetch(`${SCREENPIPE_URL}/health`);
        if (!cancelled && res.ok) {
          setStatus({ connected: true, last_frame_at: Date.now() });
        }
      } catch {
        if (!cancelled) {
          setStatus((s) => ({ ...s, connected: false }));
        }
      }
    }

    probe();
    const id = setInterval(probe, 5000);
    return () => {
      cancelled = true;
      clearInterval(id);
    };
  }, []);

  return status;
}
