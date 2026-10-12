import { invoke } from "@tauri-apps/api/core";
import { eq } from "drizzle-orm";
import { db, profiles, saveSets } from "@/db";
import type { ProfileChange, ProfileList } from "@/lib/types";

/** Rust reconciles file inventory; Drizzle provides typed identity and ownership reads. */
export const profilesService = {
  async list(): Promise<ProfileList> {
    const inventory = await invoke<ProfileList>("list_profiles");
    if (inventory.setup_required) return inventory;
    const rows = await db.select({ id: profiles.id, name: profiles.name, path: saveSets.userDir })
      .from(profiles).innerJoin(saveSets, eq(saveSets.profileId, profiles.id));
    return { ...inventory, profiles: inventory.profiles.map((profile) => {
      const owner = rows.find((row) => row.id === profile.id);
      if (!owner || owner.path !== profile.path) throw new Error("Save ownership is inconsistent.");
      return { ...profile, name: owner.name };
    }) };
  },
  change: (change: ProfileChange) => invoke<void>("change_profile", change),
};
