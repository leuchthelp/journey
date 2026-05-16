import { integer, sqliteTable, text } from "drizzle-orm/sqlite-core";

export const mediaItems = sqliteTable("MediaItems", {
  id: integer("id").primaryKey().unique(),
  hash: text("hash").unique().default(""),
  backgroundImage: text("backgroundImage").default(""),
  content: text("content").default(""),
  outlineGradient: text("outlineGradient").default("ring-[#C2381D]"),
  defaultStyling: text("defaultStyling").default(
    "m-0.5 h-24 w-24 rounded-full bg-amber-200 ring",
  ),
  animation: text("animation").default(""),
  loaded: integer("loaded", { mode: "boolean" }).default(false),
  local: text("local").default(""),
  providers: text("providers").default('{"test": "location"}'),
});

export type MediaItems = typeof mediaItems.$inferSelect;
