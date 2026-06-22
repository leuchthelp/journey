import { db } from "./database.ts";
import { sql } from "drizzle-orm";

const singlePageDataQuery = db.query.mediaItems
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

const mainPageDataQuery = db.query.mediaItems
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

const providerDataQuery = db.query.providerItems.findMany().prepare();

export { singlePageDataQuery, mainPageDataQuery, providerDataQuery };
