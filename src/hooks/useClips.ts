import { useState, useEffect, useCallback } from "react";
import { getClips, deleteClip, renameClip } from "../services/api";
import type { Clip, ClipSortField, ClipSortOrder } from "../types";

export function useClips(
  sortField: ClipSortField = "date",
  sortOrder: ClipSortOrder = "desc",
  search = ""
) {
  const [clips, setClips] = useState<Clip[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const data = await getClips(sortField, sortOrder, search);
      setClips(data);
      setError(null);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, [sortField, sortOrder, search]);

  useEffect(() => { load(); }, [load]);

  const remove = useCallback(async (id: string) => {
    await deleteClip(id);
    setClips((prev) => prev.filter((c) => c.id !== id));
  }, []);

  const rename = useCallback(async (id: string, name: string) => {
    await renameClip(id, name);
    await load();
  }, [load]);

  return { clips, loading, error, refresh: load, remove, rename };
}
