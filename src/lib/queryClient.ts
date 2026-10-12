import { QueryClient } from "@tanstack/react-query";

// Backend data is local and changes only through this app (or the game's menu, between
// starts): no polling, no refetch on focus.
export const queryClient = new QueryClient({
  defaultOptions: {
    // Error views also observe these queries. Retry-on-mount would remount them in a loop.
    queries: { staleTime: Infinity, refetchOnWindowFocus: false, retry: false, retryOnMount: false },
  },
});

export const queryKeys = {
  profiles: ["profiles"] as const,
  saves: ["saves"] as const,
  config: ["config"] as const,
  gameCheck: (dir: string) => ["game-check", dir] as const,
  mods: ["mods"] as const,
  patches: ["patches"] as const,
  gamepads: ["gamepads"] as const,
  displays: ["displays"] as const,
};
