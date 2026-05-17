PRAGMA foreign_keys=OFF;--> statement-breakpoint
CREATE TABLE `__new_MediaItems` (
	`id` integer PRIMARY KEY NOT NULL,
	`hash` text DEFAULT '' NOT NULL,
	`type` text DEFAULT 'MediaItem' NOT NULL,
	`backgroundImage` text DEFAULT '' NOT NULL,
	`content` text DEFAULT '' NOT NULL,
	`outlineGradient` text DEFAULT 'ring-[#C2381D]' NOT NULL,
	`defaultStyling` text DEFAULT 'm-0.5 h-24 w-24 rounded-full bg-amber-200 ring' NOT NULL,
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