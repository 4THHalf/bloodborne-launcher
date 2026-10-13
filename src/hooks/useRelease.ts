import { useQuery } from "@tanstack/react-query";
import { api, queryKeys } from "@/lib";
import { useConfig } from "@/hooks/useConfig";

const SIX_HOURS = 6 * 60 * 60 * 1000;

/** Shared across the banner and settings; failures never block local launcher data. */
export function useRelease() {
  const { data: config } = useConfig();
  const automatic = !!config && (config.settings.check_updates ?? true);
  return useQuery({
    queryKey: queryKeys.release,
    queryFn: api.checkRelease,
    enabled: automatic,
    staleTime: SIX_HOURS,
    refetchInterval: automatic ? SIX_HOURS : false,
  });
}
