CREATE TABLE `ContentItems` (
	`id` integer PRIMARY KEY,
	`parentId` integer NOT NULL,
	`type` text DEFAULT '',
	`description` text
);
--> statement-breakpoint
CREATE TABLE `ImageItems` (
	`id` integer PRIMARY KEY,
	`providerId` text DEFAULT '' NOT NULL,
	`type` text DEFAULT '' NOT NULL,
	`url` text DEFAULT '' NOT NULL
);
--> statement-breakpoint
CREATE TABLE `MediaItemChildren` (
	`parentId` integer NOT NULL,
	`childId` integer NOT NULL,
	CONSTRAINT `MediaItemChildren_pk` PRIMARY KEY(`parentId`, `childId`),
	CONSTRAINT `fk_MediaItemChildren_parentId_MediaItems_id_fk` FOREIGN KEY (`parentId`) REFERENCES `MediaItems`(`id`),
	CONSTRAINT `fk_MediaItemChildren_childId_MediaItems_id_fk` FOREIGN KEY (`childId`) REFERENCES `MediaItems`(`id`)
);
--> statement-breakpoint
CREATE TABLE `MediaItemToImageItem` (
	`mediaItemId` integer NOT NULL,
	`imageItemId` integer NOT NULL,
	CONSTRAINT `MediaItemToImageItem_pk` PRIMARY KEY(`mediaItemId`, `imageItemId`),
	CONSTRAINT `fk_MediaItemToImageItem_mediaItemId_MediaItems_id_fk` FOREIGN KEY (`mediaItemId`) REFERENCES `MediaItems`(`id`),
	CONSTRAINT `fk_MediaItemToImageItem_imageItemId_ImageItems_id_fk` FOREIGN KEY (`imageItemId`) REFERENCES `ImageItems`(`id`)
);
--> statement-breakpoint
CREATE TABLE `ProviderItemToImageItem` (
	`mediaItemId` integer NOT NULL,
	`providerItemId` integer NOT NULL,
	CONSTRAINT `ProviderItemToImageItem_pk` PRIMARY KEY(`mediaItemId`, `providerItemId`),
	CONSTRAINT `fk_ProviderItemToImageItem_mediaItemId_MediaItems_id_fk` FOREIGN KEY (`mediaItemId`) REFERENCES `MediaItems`(`id`),
	CONSTRAINT `fk_ProviderItemToImageItem_providerItemId_ProviderItems_id_fk` FOREIGN KEY (`providerItemId`) REFERENCES `ProviderItems`(`id`)
);
--> statement-breakpoint
ALTER TABLE `MediaItems` RENAME COLUMN `hash` TO `uuid`;--> statement-breakpoint
ALTER TABLE `ProviderItems` ADD `id` integer;--> statement-breakpoint
PRAGMA foreign_keys=OFF;--> statement-breakpoint
CREATE TABLE `__new_ProviderItems` (
	`id` integer PRIMARY KEY,
	`userId` text DEFAULT '' NOT NULL UNIQUE,
	`serverId` text DEFAULT '' NOT NULL,
	`type` text DEFAULT '' NOT NULL,
	`url` text DEFAULT '' NOT NULL
);
--> statement-breakpoint
INSERT INTO `__new_ProviderItems`(`userId`, `serverId`, `url`, `type`) SELECT `userId`, `serverId`, `url`, `type` FROM `ProviderItems`;--> statement-breakpoint
DROP TABLE `ProviderItems`;--> statement-breakpoint
ALTER TABLE `__new_ProviderItems` RENAME TO `ProviderItems`;--> statement-breakpoint
PRAGMA foreign_keys=ON;--> statement-breakpoint
PRAGMA foreign_keys=OFF;--> statement-breakpoint
CREATE TABLE `__new_MediaItems` (
	`id` integer PRIMARY KEY,
	`uuid` text DEFAULT '',
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
PRAGMA foreign_keys=ON;--> statement-breakpoint
DROP INDEX IF EXISTS `MediaItems_id_unique`;--> statement-breakpoint
DROP INDEX IF EXISTS `MediaItems_hash_unique`;--> statement-breakpoint
DROP INDEX IF EXISTS `ProviderItems_userId_unique`;--> statement-breakpoint
DROP INDEX IF EXISTS `account_userId_idx`;--> statement-breakpoint
DROP INDEX IF EXISTS `session_token_unique`;--> statement-breakpoint
DROP INDEX IF EXISTS `session_userId_idx`;--> statement-breakpoint
DROP INDEX IF EXISTS `user_email_unique`;--> statement-breakpoint
DROP INDEX IF EXISTS `verification_identifier_idx`;--> statement-breakpoint
CREATE INDEX `ContentToItemId_idx` ON `ContentItems` (`parentId`);--> statement-breakpoint
CREATE INDEX `ImageProviderId_idx` ON `ImageItems` (`providerId`);--> statement-breakpoint
CREATE INDEX `ParentsToChildrenParentId_idx` ON `MediaItemChildren` (`parentId`);--> statement-breakpoint
CREATE INDEX `ParentsToChildrenChildId_idx` ON `MediaItemChildren` (`childId`);--> statement-breakpoint
CREATE INDEX `ParentsToChildrenCompositeId_idx` ON `MediaItemChildren` (`parentId`,`childId`);--> statement-breakpoint
CREATE INDEX `ItemToImagesItemID_idx` ON `MediaItemToImageItem` (`mediaItemId`);--> statement-breakpoint
CREATE INDEX `ItemToImagesImageID_idx` ON `MediaItemToImageItem` (`imageItemId`);--> statement-breakpoint
CREATE INDEX `ItemToImagesCompositeID_idx` ON `MediaItemToImageItem` (`mediaItemId`,`imageItemId`);--> statement-breakpoint
CREATE INDEX `ItemToProvidersItemId_idx` ON `ProviderItemToImageItem` (`mediaItemId`);--> statement-breakpoint
CREATE INDEX `ItemToProvidersProviderId_idx` ON `ProviderItemToImageItem` (`providerItemId`);--> statement-breakpoint
CREATE INDEX `ItemToProvidersCompositeId_idx` ON `ProviderItemToImageItem` (`mediaItemId`,`providerItemId`);--> statement-breakpoint
DROP TABLE `account`;--> statement-breakpoint
DROP TABLE `session`;--> statement-breakpoint
DROP TABLE `user`;--> statement-breakpoint
DROP TABLE `verification`;