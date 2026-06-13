import {
  serial,
  boolean,
  pgTable,
  text,
  primaryKey,
  index,
} from "drizzle-orm/pg-core";

export const mediaItems = pgTable("MediaItems", {
  uuid: text("uuid").primaryKey().unique(),
  type: text("type").default("MediaItem").notNull(),
  outlineGradient: text("outlineGradient").default("ring-[#C2381D]").notNull(),
  loaded: boolean("loaded").default(false).notNull(),
  local: text("local").default("").notNull(),
});

export const mediaItemChildren = pgTable(
  "MediaItemChildren",
  {
    parentId: text("parentId")
      .notNull()
      .references(() => mediaItems.uuid),
    childId: text("childId")
      .notNull()
      .references(() => mediaItems.uuid),
  },
  (t) => [
    primaryKey({ columns: [t.parentId, t.childId] }),
    index("ParentsToChildrenParentId_idx").on(t.parentId),
    index("ParentsToChildrenChildId_idx").on(t.childId),
    index("ParentsToChildrenCompositeId_idx").on(t.parentId, t.childId),
  ],
);

/*

MediaItem OriginalItem Mapping

*/

export const originalItems = pgTable(
  "OriginalItems",
  {
    id: serial("id").primaryKey(),
    parentId: text("parentId").notNull(),
    serverId: text("serverId").notNull(),
    uuid: text("uuid").notNull(),
    url: text("url").notNull()
  },
  (t) => [index("OrignalToItemId_idx").on(t.parentId)],
);

/*

MediaItem Content Mapping

*/

export const contentItems = pgTable(
  "ContentItems",
  {
    id: serial("id").primaryKey(),
    parentId: text("parentId").notNull(),
    type: text("type").notNull(),
    description: text("description").notNull(),
  },
  (t) => [index("ContentToItemId_idx").on(t.parentId)],
);

/*

Provider Mapping

*/

export const providerItems = pgTable("ProviderItems", {
  userId: text("userId").primaryKey().unique().notNull(),
  serverId: text("serverId").default("").notNull(),
  type: text("type").default("").notNull(),
  url: text("url").default("").notNull(),
});

export const mediaItemToProviderItem = pgTable(
  "ProviderItemToImageItem",
  {
    mediaItemId: text("mediaItemId")
      .notNull()
      .references(() => mediaItems.uuid),
    providerItemId: text("providerItemId")
      .notNull()
      .references(() => providerItems.userId),
  },
  (t) => [
    primaryKey({ columns: [t.mediaItemId, t.providerItemId] }),
    index("ItemToProvidersItemId_idx").on(t.mediaItemId),
    index("ItemToProvidersProviderId_idx").on(t.providerItemId),
    index("ItemToProvidersCompositeId_idx").on(t.mediaItemId, t.providerItemId),
  ],
);

/*

Image Mapping

*/

export const imageItems = pgTable(
  "ImageItems",
  {
    url: text("url").primaryKey().unique().notNull(),
    serverId: text("serverId").default("").notNull(),
    type: text("type").notNull(),
  },
  (t) => [index("ImageProviderId_idx").on(t.serverId)],
);

export const mediaItemToImageItem = pgTable(
  "MediaItemToImageItem",
  {
    mediaItemId: text("mediaItemId")
      .notNull()
      .references(() => mediaItems.uuid),
    imageItemId: text("imageItemId")
      .notNull()
      .references(() => imageItems.url),
  },
  (t) => [
    primaryKey({ columns: [t.mediaItemId, t.imageItemId] }),
    index("ItemToImagesItemID_idx").on(t.mediaItemId),
    index("ItemToImagesImageID_idx").on(t.imageItemId),
    index("ItemToImagesCompositeID_idx").on(t.mediaItemId, t.imageItemId),
  ],
);

/*

Type Exports

*/

export type OriginalItem = Omit<typeof originalItems.$inferSelect, "id">;
export type ContentItem = Omit<typeof contentItems.$inferSelect, "id">;
export type ProviderItem = typeof providerItems.$inferSelect;
export type ImageItem = typeof imageItems.$inferSelect;
