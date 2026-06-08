CREATE TABLE `ContentItems` (
	`id` integer PRIMARY KEY,
	`parentId` text NOT NULL,
	`type` text NOT NULL,
	`description` text NOT NULL
);
--> statement-breakpoint
CREATE TABLE `ImageItems` (
	`url` text PRIMARY KEY UNIQUE,
	`serverId` text DEFAULT '' NOT NULL,
	`type` text NOT NULL
);
--> statement-breakpoint
CREATE TABLE `MediaItemChildren` (
	`parentId` text NOT NULL,
	`childId` text NOT NULL,
	CONSTRAINT `MediaItemChildren_pk` PRIMARY KEY(`parentId`, `childId`),
	CONSTRAINT `fk_MediaItemChildren_parentId_MediaItems_uuid_fk` FOREIGN KEY (`parentId`) REFERENCES `MediaItems`(`uuid`),
	CONSTRAINT `fk_MediaItemChildren_childId_MediaItems_uuid_fk` FOREIGN KEY (`childId`) REFERENCES `MediaItems`(`uuid`)
);
--> statement-breakpoint
CREATE TABLE `MediaItemToImageItem` (
	`mediaItemId` text NOT NULL,
	`imageItemId` text NOT NULL,
	CONSTRAINT `MediaItemToImageItem_pk` PRIMARY KEY(`mediaItemId`, `imageItemId`),
	CONSTRAINT `fk_MediaItemToImageItem_mediaItemId_MediaItems_uuid_fk` FOREIGN KEY (`mediaItemId`) REFERENCES `MediaItems`(`uuid`),
	CONSTRAINT `fk_MediaItemToImageItem_imageItemId_ImageItems_url_fk` FOREIGN KEY (`imageItemId`) REFERENCES `ImageItems`(`url`)
);
--> statement-breakpoint
CREATE TABLE `ProviderItemToImageItem` (
	`mediaItemId` text NOT NULL,
	`providerItemId` text NOT NULL,
	CONSTRAINT `ProviderItemToImageItem_pk` PRIMARY KEY(`mediaItemId`, `providerItemId`),
	CONSTRAINT `fk_ProviderItemToImageItem_mediaItemId_MediaItems_uuid_fk` FOREIGN KEY (`mediaItemId`) REFERENCES `MediaItems`(`uuid`),
	CONSTRAINT `fk_ProviderItemToImageItem_providerItemId_ProviderItems_userId_fk` FOREIGN KEY (`providerItemId`) REFERENCES `ProviderItems`(`userId`)
);
--> statement-breakpoint
CREATE TABLE `MediaItems` (
	`uuid` text PRIMARY KEY UNIQUE,
	`type` text DEFAULT 'MediaItem' NOT NULL,
	`outlineGradient` text DEFAULT 'ring-[#C2381D]' NOT NULL,
	`defaultStyling` text DEFAULT 'm-0.5 h-24 w-24 rounded-xl bg-amber-200 ring' NOT NULL,
	`loaded` integer DEFAULT false NOT NULL,
	`local` text DEFAULT '' NOT NULL
);
--> statement-breakpoint
CREATE TABLE `OriginalItems` (
	`id` integer PRIMARY KEY,
	`parentId` text NOT NULL,
	`serverId` text NOT NULL,
	`uuid` text NOT NULL
);
--> statement-breakpoint
CREATE TABLE `ProviderItems` (
	`userId` text PRIMARY KEY UNIQUE,
	`serverId` text DEFAULT '' NOT NULL,
	`type` text DEFAULT '' NOT NULL,
	`url` text DEFAULT '' NOT NULL
);
--> statement-breakpoint
CREATE INDEX `ContentToItemId_idx` ON `ContentItems` (`parentId`);--> statement-breakpoint
CREATE INDEX `ImageProviderId_idx` ON `ImageItems` (`serverId`);--> statement-breakpoint
CREATE INDEX `ParentsToChildrenParentId_idx` ON `MediaItemChildren` (`parentId`);--> statement-breakpoint
CREATE INDEX `ParentsToChildrenChildId_idx` ON `MediaItemChildren` (`childId`);--> statement-breakpoint
CREATE INDEX `ParentsToChildrenCompositeId_idx` ON `MediaItemChildren` (`parentId`,`childId`);--> statement-breakpoint
CREATE INDEX `ItemToImagesItemID_idx` ON `MediaItemToImageItem` (`mediaItemId`);--> statement-breakpoint
CREATE INDEX `ItemToImagesImageID_idx` ON `MediaItemToImageItem` (`imageItemId`);--> statement-breakpoint
CREATE INDEX `ItemToImagesCompositeID_idx` ON `MediaItemToImageItem` (`mediaItemId`,`imageItemId`);--> statement-breakpoint
CREATE INDEX `ItemToProvidersItemId_idx` ON `ProviderItemToImageItem` (`mediaItemId`);--> statement-breakpoint
CREATE INDEX `ItemToProvidersProviderId_idx` ON `ProviderItemToImageItem` (`providerItemId`);--> statement-breakpoint
CREATE INDEX `ItemToProvidersCompositeId_idx` ON `ProviderItemToImageItem` (`mediaItemId`,`providerItemId`);--> statement-breakpoint
CREATE INDEX `OrignalToItemId_idx` ON `OriginalItems` (`parentId`);