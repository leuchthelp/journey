CREATE TABLE `account` (
	`id` text PRIMARY KEY NOT NULL,
	`account_id` text NOT NULL,
	`provider_id` text NOT NULL,
	`user_id` text NOT NULL,
	`access_token` text,
	`refresh_token` text,
	`id_token` text,
	`access_token_expires_at` integer,
	`refresh_token_expires_at` integer,
	`scope` text,
	`password` text,
	`created_at` integer DEFAULT (cast(unixepoch('subsecond') * 1000 as integer)) NOT NULL,
	`updated_at` integer NOT NULL,
	FOREIGN KEY (`user_id`) REFERENCES `user`(`id`) ON UPDATE no action ON DELETE cascade
);
--> statement-breakpoint
CREATE INDEX `account_userId_idx` ON `account` (`user_id`);--> statement-breakpoint
CREATE TABLE `session` (
	`id` text PRIMARY KEY NOT NULL,
	`expires_at` integer NOT NULL,
	`token` text NOT NULL,
	`created_at` integer DEFAULT (cast(unixepoch('subsecond') * 1000 as integer)) NOT NULL,
	`updated_at` integer NOT NULL,
	`ip_address` text,
	`user_agent` text,
	`user_id` text NOT NULL,
	FOREIGN KEY (`user_id`) REFERENCES `user`(`id`) ON UPDATE no action ON DELETE cascade
);
--> statement-breakpoint
CREATE UNIQUE INDEX `session_token_unique` ON `session` (`token`);--> statement-breakpoint
CREATE INDEX `session_userId_idx` ON `session` (`user_id`);--> statement-breakpoint
CREATE TABLE `user` (
	`id` text PRIMARY KEY NOT NULL,
	`name` text NOT NULL,
	`email` text NOT NULL,
	`email_verified` integer DEFAULT false NOT NULL,
	`image` text,
	`created_at` integer DEFAULT (cast(unixepoch('subsecond') * 1000 as integer)) NOT NULL,
	`updated_at` integer DEFAULT (cast(unixepoch('subsecond') * 1000 as integer)) NOT NULL
);
--> statement-breakpoint
CREATE UNIQUE INDEX `user_email_unique` ON `user` (`email`);--> statement-breakpoint
CREATE TABLE `verification` (
	`id` text PRIMARY KEY NOT NULL,
	`identifier` text NOT NULL,
	`value` text NOT NULL,
	`expires_at` integer NOT NULL,
	`created_at` integer DEFAULT (cast(unixepoch('subsecond') * 1000 as integer)) NOT NULL,
	`updated_at` integer DEFAULT (cast(unixepoch('subsecond') * 1000 as integer)) NOT NULL
);
--> statement-breakpoint
CREATE INDEX `verification_identifier_idx` ON `verification` (`identifier`);--> statement-breakpoint
PRAGMA foreign_keys=OFF;--> statement-breakpoint
CREATE TABLE `__new_MediaItems` (
	`id` integer PRIMARY KEY NOT NULL,
	`hash` text DEFAULT '' NOT NULL,
	`type` text DEFAULT 'MediaItem' NOT NULL,
	`backgroundImage` text DEFAULT '' NOT NULL,
	`content` text DEFAULT '' NOT NULL,
	`outlineGradient` text DEFAULT 'ring-[#C2381D]' NOT NULL,
	`defaultStyling` text DEFAULT 'm-0.5 h-24 w-24 rounded-xl bg-amber-200 ring' NOT NULL,
	`animation` text DEFAULT '' NOT NULL,
	`loaded` integer DEFAULT false NOT NULL,
	`local` text DEFAULT '' NOT NULL,
	`providers` text DEFAULT '{"test": "location"}' NOT NULL
);
--> statement-breakpoint
INSERT INTO `__new_MediaItems`("id", "hash", "type", "backgroundImage", "content", "outlineGradient", "defaultStyling", "animation", "loaded", "local", "providers") SELECT "id", "hash", "type", "backgroundImage", "content", "outlineGradient", "defaultStyling", "animation", "loaded", "local", "providers" FROM `MediaItems`;--> statement-breakpoint
DROP TABLE `MediaItems`;--> statement-breakpoint
ALTER TABLE `__new_MediaItems` RENAME TO `MediaItems`;--> statement-breakpoint
PRAGMA foreign_keys=ON;--> statement-breakpoint
CREATE UNIQUE INDEX `MediaItems_id_unique` ON `MediaItems` (`id`);--> statement-breakpoint
CREATE UNIQUE INDEX `MediaItems_hash_unique` ON `MediaItems` (`hash`);