PRAGMA foreign_keys=OFF;--> statement-breakpoint
CREATE TABLE `__new_ProviderItems` (
	`userId` text PRIMARY KEY DEFAULT '' NOT NULL,
	`serverId` text DEFAULT '' NOT NULL,
	`url` text DEFAULT '' NOT NULL,
	`type` text DEFAULT '' NOT NULL
);
--> statement-breakpoint
INSERT INTO `__new_ProviderItems`("userId", "serverId", "url", "type") SELECT "userId", "serverId", "url", "type" FROM `ProviderItems`;--> statement-breakpoint
DROP TABLE `ProviderItems`;--> statement-breakpoint
ALTER TABLE `__new_ProviderItems` RENAME TO `ProviderItems`;--> statement-breakpoint
PRAGMA foreign_keys=ON;--> statement-breakpoint
DROP INDEX `MediaItems_id_unique`;