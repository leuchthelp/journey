import { db } from "./database.ts";
import { sql } from "drizzle-orm";

export const singlePageDataQuery = db.query.mediaItems
  .findFirst({
    where: { uuid: sql.placeholder("slug") },
    with: {
      original: { columns: { id: false } },
      content: { columns: { id: false } },
      providers: true,
      images: true,
      parents: {
        columns: { uuid: true },
      },
    },
  })
  .prepare();

export const mainPageDataQuery = db.query.mediaItems
  .findMany({
    limit: sql.placeholder("limit"),
    where: { type: sql.placeholder("type") },
    with: {
      original: { columns: { id: false } },
      content: { columns: { id: false } },
      providers: true,
      images: true,
      parents: { columns: { uuid: true } },
    },
  })
  .prepare();

export const providerDataQuery = db.query.providerItems.findMany().prepare();
