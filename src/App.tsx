import { ProfileSetup } from "@/components/profiles";
import { AppWindow, LoadingScreen } from "@/components/window";
import { useConfig, useProfiles, useGameConnection, useLanguageSync } from "@/hooks";

export function App() {
  const { data } = useConfig();
  const profiles = useProfiles();
  useLanguageSync();
  useGameConnection();
  if (!data || profiles.isPending) return <LoadingScreen />;
  return profiles.error || profiles.data?.setup_required ? <ProfileSetup /> : <AppWindow />;
}
