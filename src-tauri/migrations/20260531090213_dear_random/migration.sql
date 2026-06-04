CREATE TABLE `ServerItems` (
	`id` integer PRIMARY KEY NOT NULL,
	`url` text DEFAULT '' NOT NULL
);
--> statement-breakpoint
CREATE UNIQUE INDEX `ServerItems_id_unique` ON `ServerItems` (`id`);