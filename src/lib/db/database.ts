import { PGlite } from "@electric-sql/pglite";
import { drizzle } from "drizzle-orm/pglite";
import { relations } from "./relations.ts";

const client = new PGlite("idb://dev");
const db = drizzle({ client: client, relations: relations });

export { db };
