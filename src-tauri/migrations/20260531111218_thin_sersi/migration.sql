ALTER TABLE `ServerItems` RENAME TO `ProviderItems`;--> statement-breakpoint
DROP INDEX `ServerItems_id_unique`;--> statement-breakpoint
CREATE UNIQUE INDEX `ProviderItems_id_unique` ON `ProviderItems` (`id`);