CREATE TABLE `MediaItems` (
	`id` integer PRIMARY KEY NOT NULL,
	`hash` text DEFAULT '',
	`backgroundImage` text DEFAULT '',
	`content` text DEFAULT '',
	`outlineGradient` text DEFAULT 'ring-[#C2381D]',
	`defaultStyling` text DEFAULT 'm-0.5 h-24 w-24 rounded-full bg-amber-200 ring',
	`animation` text DEFAULT '',
	`loaded` integer DEFAULT false,
	`local` text DEFAULT '',
	`providers` text DEFAULT '{"test": "location"}'
);
--> statement-breakpoint
CREATE UNIQUE INDEX `MediaItems_id_unique` ON `MediaItems` (`id`);--> statement-breakpoint
CREATE UNIQUE INDEX `MediaItems_hash_unique` ON `MediaItems` (`hash`);