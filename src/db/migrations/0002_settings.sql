CREATE TABLE `settings` (
	`scope` text NOT NULL,
	`key` text NOT NULL,
	`value` text NOT NULL,
	`updated_at` integer NOT NULL,
	PRIMARY KEY(`scope`, `key`),
	CONSTRAINT "settings_scope" CHECK("settings"."scope" IN ('launcher', 'game'))
);
