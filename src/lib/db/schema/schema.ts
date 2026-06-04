import { integer, sqliteTable, text } from "drizzle-orm/sqlite-core";

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

export const imageItems = sqliteTable("ImageItems", {
  id: integer("id").primaryKey(),
  ownerId: text("ownerId").default("").notNull(),
  provider: text("provider").default("").notNull(),
  type: text("type").default("").notNull(),
  url: text("url").default("").notNull(),
});

export const providerItems = sqliteTable("ProviderItems", {
  id: integer("id").primaryKey(),
  userId: text("userId").default("").unique().notNull(),
  serverId: text("serverId").default("").notNull(),
  type: text("type").default("").notNull(),
  url: text("url").default("").notNull(),
});

export const contentItems = sqliteTable("ContentItems", {
  id: integer("id").primaryKey()
});

export type MediaItem = typeof mediaItems.$inferSelect;
export type ImageItems = typeof imageItems.$inferSelect;
export type ProviderItem = typeof providerItems.$inferSelect;
export type ContentItems = typeof contentItems.$inferSelect;
