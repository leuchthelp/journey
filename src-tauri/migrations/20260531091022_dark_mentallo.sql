PRAGMA foreign_keys=OFF;--> statement-breakpoint
CREATE TABLE `__new_ServerItems` (
	`id` text PRIMARY KEY NOT NULL,
	`url` text DEFAULT '' NOT NULL
);
--> statement-breakpoint
INSERT INTO `__new_ServerItems`("id", "url") SELECT "id", "url" FROM `ServerItems`;--> statement-breakpoint
DROP TABLE `ServerItems`;--> statement-breakpoint
ALTER TABLE `__new_ServerItems` RENAME TO `ServerItems`;--> statement-breakpoint
PRAGMA foreign_keys=ON;--> statement-breakpoint
CREATE UNIQUE INDEX `ServerItems_id_unique` ON `ServerItems` (`id`);