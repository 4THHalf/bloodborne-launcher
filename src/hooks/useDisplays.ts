import { useQuery } from "@tanstack/react-query";
import { api, queryKeys } from "@/lib";

/** Connected monitors (name, size, main one); refetch to pick up new ones. */
export function useDisplays() {
  return useQuery({ queryKey: queryKeys.displays, queryFn: api.listDisplays });
}
