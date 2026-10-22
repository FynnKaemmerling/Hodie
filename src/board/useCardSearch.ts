import { useEffect, useState } from "react";
import { api, type Id } from "../api";
import { useNotifications } from "../notifications/Notifications";

const SEARCH_DELAY_MS = 150;

/**
 * Ids of cards whose title or notes match `query`, searched shortly after
 * typing stops. `null` while there is no query. `revision` re-runs the search
 * when cards change.
 */
export function useCardSearch(boardId: Id | null, query: string, revision: unknown) {
  const notify = useNotifications();
  const [matches, setMatches] = useState<ReadonlySet<Id> | null>(null);
  const trimmed = query.trim();

  useEffect(() => {
    if (boardId === null || trimmed === "") {
      setMatches(null);
      return;
    }
    let cancelled = false;
    const timer = window.setTimeout(async () => {
      try {
        const ids = await api.searchCards(boardId, trimmed);
        if (!cancelled) setMatches(new Set(ids));
      } catch (error) {
        notify.error(error);
      }
    }, SEARCH_DELAY_MS);
    return () => {
      cancelled = true;
      window.clearTimeout(timer);
    };
  }, [boardId, trimmed, revision, notify]);

  return matches;
}
