import { db } from "./database";
import { sql } from "drizzle-orm";

export const singlePageDataQuery = db.query.mediaItems
  .findFirst({
    where: { uuid: sql.placeholder("slug") },
    columns: { id: false },
    with: {
      content: { columns: { id: false } },
      providers: { columns: { id: false } },
      images: { columns: { id: false } },
      parents: {
        columns: { uuid: true },
      },
    },
  })
  .prepare();

export const mainPageDataQuery = db.query.mediaItems
  .findMany({
    limit: sql.placeholder("limit"),
    columns: { id: false },
    with: {
      content: { columns: { id: false } },
      providers: { columns: { id: false } },
      images: { columns: { id: false } },
      parents: { columns: { uuid: true } },
    },
  })
  .prepare();

export const providerDataQuery = db.query.providerItems
  .findMany({
    columns: { id: false },
  })
  .prepare();
