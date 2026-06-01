ALTER TABLE `ProviderItems` ADD `user` text DEFAULT '' NOT NULL;--> statement-breakpoint
CREATE UNIQUE INDEX `ProviderItems_user_unique` ON `ProviderItems` (`user`);