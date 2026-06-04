import {
  integer,
  sqliteTable,
  text,
  primaryKey,
  index,
} from "drizzle-orm/sqlite-core";

export const mediaItems = sqliteTable("MediaItems", {
  id: integer("id").primaryKey(),
  uuid: text("uuid").default(""),
  type: text("type").default("MediaItem").notNull(),
  outlineGradient: text("outlineGradient").default("ring-[#C2381D]").notNull(),
  defaultStyling: text("defaultStyling")
    .default("m-0.5 h-24 w-24 rounded-xl bg-amber-200 ring")
    .notNull(),
  loaded: integer("loaded", { mode: "boolean" }).default(false).notNull(),
  local: text("local").default(""),
});

export const mediaItemChildren = sqliteTable(
  "MediaItemChildren",
  {
    parentId: integer("parentId")
      .notNull()
      .references(() => mediaItems.id),
    childId: integer("childId")
      .notNull()
      .references(() => mediaItems.id),
  },
  (t) => [
    primaryKey({ columns: [t.parentId, t.childId] }),
    index("ParentsToChildrenParentId_idx").on(t.parentId),
    index("ParentsToChildrenChildId_idx").on(t.childId),
    index("ParentsToChildrenCompositeId_idx").on(t.parentId, t.childId),
  ],
);

/*

MediaItem Content Mapping

*/

export const contentItems = sqliteTable(
  "ContentItems",
  {
    id: integer("id").primaryKey(),
    parentId: integer("parentId").notNull(),
    type: text("type").default(""),
    description: text("description"),
  },
  (table) => [index("ContentToItemId_idx").on(table.parentId)],
);

/*

Provider Mapping

*/

export const providerItems = sqliteTable("ProviderItems", {
  id: integer("id").primaryKey(),
  userId: text("userId").default("").unique().notNull(),
  serverId: text("serverId").default("").notNull(),
  type: text("type").default("").notNull(),
  url: text("url").default("").notNull(),
});

export const mediaItemToProviderItem = sqliteTable(
  "ProviderItemToImageItem",
  {
    mediaItemId: integer("mediaItemId")
      .notNull()
      .references(() => mediaItems.id),
    providerItemId: integer("providerItemId")
      .notNull()
      .references(() => providerItems.id),
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

export const imageItems = sqliteTable(
  "ImageItems",
  {
    id: integer("id").primaryKey(),
    providerId: text("providerId").default("").notNull(),
    type: text("type").default("").notNull(),
    url: text("url").default("").notNull(),
  },
  (t) => [index("ImageProviderId_idx").on(t.providerId)],
);

export const mediaItemToImageItem = sqliteTable(
  "MediaItemToImageItem",
  {
    mediaItemId: integer("mediaItemId")
      .notNull()
      .references(() => mediaItems.id),
    imageItemId: integer("imageItemId")
      .notNull()
      .references(() => imageItems.id),
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

export type MediaItem = typeof mediaItems.$inferSelect;
export type ContentItems = typeof contentItems.$inferSelect;
export type ProviderItem = typeof providerItems.$inferSelect;
export type ImageItems = typeof imageItems.$inferSelect;
