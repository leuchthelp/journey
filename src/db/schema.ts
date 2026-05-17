import { integer, sqliteTable, text } from "drizzle-orm/sqlite-core";

export const mediaItems = sqliteTable("MediaItems", {
  id: integer("id").primaryKey().unique(),
  hash: text("hash").unique().default("").notNull(),
  type: text("type").default("MediaItem").notNull(),
  backgroundImage: text("backgroundImage").default("").notNull(),
  content: text("content").default("").notNull(),
  outlineGradient: text("outlineGradient").default("ring-[#C2381D]").notNull(),
  defaultStyling: text("defaultStyling")
    .default("m-0.5 h-24 w-24 rounded-xl bg-amber-200 ring")
    .notNull(),
  animation: text("animation").default("").notNull(),
  loaded: integer("loaded", { mode: "boolean" }).default(false).notNull(),
  local: text("local").default("").notNull(),
  providers: text("providers").default('{"test": "location"}').notNull(),
});

export type MediaItems = typeof mediaItems.$inferSelect;
