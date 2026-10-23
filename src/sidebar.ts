import { useCallback, useState } from "react";

// View state like the last open board, so it lives in localStorage too.
const STORAGE_KEY = "hodie.sidebarCollapsed";

function readCollapsed(): boolean {
  try {
    return localStorage.getItem(STORAGE_KEY) === "true";
  } catch {
    return false;
  }
}

/** Whether the sidebar is collapsed, remembered across restarts. */
export function useSidebarCollapsed(): [boolean, (collapsed: boolean) => void] {
  const [collapsed, setCollapsed] = useState(readCollapsed);
  const update = useCallback((value: boolean) => {
    setCollapsed(value);
    try {
      localStorage.setItem(STORAGE_KEY, String(value));
    } catch {
      // Not persisted; the sidebar still toggles for this session.
    }
  }, []);
  return [collapsed, update];
}
