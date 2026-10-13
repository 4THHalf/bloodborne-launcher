import { useEffect } from "react";
import { useIsMutating, useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { api, queryKeys } from "@/lib";
import type { ProfileChange } from "@/lib";
import { i18n } from "@/i18n";
import { useGameStore } from "@/stores";

export function useProfiles() {
  const client = useQueryClient();
  const running = useGameStore((s) => s.running);
  const query = useQuery({ queryKey: queryKeys.profiles, queryFn: api.listProfiles, refetchInterval: running ? 5000 : false });
  useEffect(() => { void client.invalidateQueries({ queryKey: queryKeys.profiles }); }, [client, running]);
  const pending = useIsMutating({ mutationKey: queryKeys.profiles });
  const mutation = useMutation({
    mutationKey: queryKeys.profiles,
    mutationFn: (change: ProfileChange) => api.changeProfile(change),
    onSuccess: async () => {
      await Promise.all([queryKeys.profiles, queryKeys.saves, queryKeys.config].map((queryKey) => client.invalidateQueries({ queryKey })));
    },
    onError: async (error, change) => {
      toast.error(i18n.t("common.saveFailed"), { description: String(error) });
      // Deletion may have committed its tombstone before a filesystem cleanup error.
      if (change.action === "delete") {
        await Promise.all([queryKeys.profiles, queryKeys.saves, queryKeys.config].map((queryKey) => client.invalidateQueries({ queryKey })));
      }
    },
  });
  return { ...query, change: mutation.mutateAsync, busy: pending > 0, running };
}
