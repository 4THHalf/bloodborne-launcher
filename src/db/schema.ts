import { sql } from "drizzle-orm";
import { check, index, integer, sqliteTable, text, uniqueIndex } from "drizzle-orm/sqlite-core";

export const profiles = sqliteTable("profiles", {
  id: text("id").primaryKey(),
  name: text("name").notNull(),
  nameKey: text("name_key").notNull().unique(),
  createdAt: integer("created_at").notNull(),
});
export const saveSets = sqliteTable("save_sets", {
  id: text("id").primaryKey(),
  profileId: text("profile_id").notNull().references(() => profiles.id, { onDelete: "restrict" }),
  userDir: text("user_dir").notNull().unique(),
}, (table) => [uniqueIndex("one_save_set_per_profile").on(table.profileId)]);
export const launcherState = sqliteTable("launcher_state", {
  id: integer("id").primaryKey(),
  activeProfileId: text("active_profile_id").notNull().references(() => profiles.id, { onDelete: "restrict" }),
  originalUserDir: text("original_user_dir").notNull(),
}, (table) => [check("singleton_state", sql`${table.id} = 1`)]);
export const backups = sqliteTable("backups", {
  id: text("id").primaryKey(),
  saveSetId: text("save_set_id").notNull().references(() => saveSets.id, { onDelete: "restrict" }),
  path: text("path").notNull().unique(),
  createdAt: integer("created_at").notNull(),
  bytes: integer("bytes").notNull(),
});
export const trophyUnlocks = sqliteTable("trophy_unlocks", {
  profileId: text("profile_id").notNull().references(() => profiles.id, { onDelete: "restrict" }),
  titleId: text("title_id").notNull(),
  trophyId: integer("trophy_id").notNull(),
  unlockedAt: integer("unlocked_at").notNull(),
}, (table) => [uniqueIndex("one_trophy_unlock").on(table.profileId, table.titleId, table.trophyId)]);
export const pendingOperations = sqliteTable("pending_operations", {
  id: text("id").primaryKey(),
  profileId: text("profile_id").references(() => profiles.id, { onDelete: "restrict" }),
  kind: text("kind").notNull(),
  status: text("status").notNull(),
  error: text("error"),
  createdAt: integer("created_at").notNull(),
}, (table) => [index("unfinished_operations").on(table.status, table.createdAt)]);
