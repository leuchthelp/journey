ALTER TABLE `ImageItems` RENAME COLUMN `providerId` TO `serverId`;--> statement-breakpoint
PRAGMA foreign_keys=OFF;--> statement-breakpoint
CREATE TABLE `__new_MediaItems` (
	`id` integer PRIMARY KEY,
	`uuid` text UNIQUE,
	`type` text DEFAULT 'MediaItem' NOT NULL,
	`outlineGradient` text DEFAULT 'ring-[#C2381D]' NOT NULL,
	`defaultStyling` text DEFAULT 'm-0.5 h-24 w-24 rounded-xl bg-amber-200 ring' NOT NULL,
	`loaded` integer DEFAULT false NOT NULL,
	`local` text DEFAULT ''
);
--> statement-breakpoint
INSERT INTO `__new_MediaItems`(`id`, `uuid`, `type`, `outlineGradient`, `defaultStyling`, `loaded`, `local`) SELECT `id`, `uuid`, `type`, `outlineGradient`, `defaultStyling`, `loaded`, `local` FROM `MediaItems`;--> statement-breakpoint
DROP TABLE `MediaItems`;--> statement-breakpoint
ALTER TABLE `__new_MediaItems` RENAME TO `MediaItems`;--> statement-breakpoint
PRAGMA foreign_keys=ON;