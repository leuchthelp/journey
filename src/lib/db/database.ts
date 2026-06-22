import { PGlite } from "@electric-sql/pglite";
import { drizzle } from "drizzle-orm/pglite";
import { relations } from "./relations.ts";
import { live } from "@electric-sql/pglite/live";

const client = new PGlite({
  dataDir: "idb://dev",
  extensions: { live },
});

const db = drizzle({ client: client, relations: relations, jit: true });

export { db, client };
