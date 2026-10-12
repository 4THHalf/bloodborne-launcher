CREATE TABLE `backups` (
	`id` text PRIMARY KEY NOT NULL,
	`save_set_id` text NOT NULL,
	`path` text NOT NULL,
	`created_at` integer NOT NULL,
	`bytes` integer NOT NULL,
	FOREIGN KEY (`save_set_id`) REFERENCES `save_sets`(`id`) ON UPDATE no action ON DELETE restrict
);
--> statement-breakpoint
CREATE UNIQUE INDEX `backups_path_unique` ON `backups` (`path`);--> statement-breakpoint
CREATE TABLE `launcher_state` (
	`id` integer PRIMARY KEY NOT NULL,
	`active_profile_id` text NOT NULL,
	`original_user_dir` text NOT NULL,
	FOREIGN KEY (`active_profile_id`) REFERENCES `profiles`(`id`) ON UPDATE no action ON DELETE restrict,
	CONSTRAINT "singleton_state" CHECK("launcher_state"."id" = 1)
);
--> statement-breakpoint
CREATE TABLE `pending_operations` (
	`id` text PRIMARY KEY NOT NULL,
	`profile_id` text,
	`kind` text NOT NULL,
	`status` text NOT NULL,
	`error` text,
	`created_at` integer NOT NULL,
	FOREIGN KEY (`profile_id`) REFERENCES `profiles`(`id`) ON UPDATE no action ON DELETE restrict
);
--> statement-breakpoint
CREATE INDEX `unfinished_operations` ON `pending_operations` (`status`,`created_at`);--> statement-breakpoint
CREATE TABLE `profiles` (
	`id` text PRIMARY KEY NOT NULL,
	`name` text NOT NULL,
	`name_key` text NOT NULL,
	`created_at` integer NOT NULL
);
--> statement-breakpoint
CREATE UNIQUE INDEX `profiles_name_key_unique` ON `profiles` (`name_key`);--> statement-breakpoint
CREATE TABLE `save_sets` (
	`id` text PRIMARY KEY NOT NULL,
	`profile_id` text NOT NULL,
	`user_dir` text NOT NULL,
	FOREIGN KEY (`profile_id`) REFERENCES `profiles`(`id`) ON UPDATE no action ON DELETE restrict
);
--> statement-breakpoint
CREATE UNIQUE INDEX `save_sets_user_dir_unique` ON `save_sets` (`user_dir`);--> statement-breakpoint
CREATE UNIQUE INDEX `one_save_set_per_profile` ON `save_sets` (`profile_id`);--> statement-breakpoint
CREATE TABLE `trophy_unlocks` (
	`profile_id` text NOT NULL,
	`title_id` text NOT NULL,
	`trophy_id` integer NOT NULL,
	`unlocked_at` integer NOT NULL,
	FOREIGN KEY (`profile_id`) REFERENCES `profiles`(`id`) ON UPDATE no action ON DELETE restrict
);
--> statement-breakpoint
CREATE UNIQUE INDEX `one_trophy_unlock` ON `trophy_unlocks` (`profile_id`,`title_id`,`trophy_id`);