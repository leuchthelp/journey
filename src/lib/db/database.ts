import { drizzle } from "drizzle-orm/sqlite-proxy";
import Database from "@tauri-apps/plugin-sql";
import { relations } from "./relations";

/**
 * Loads the sqlite database via the Tauri Proxy.
 */

async function getDb() {
  return Database.load("sqlite:dev.db");
}

/**
 * The drizzle database instance.
 */
export const db = drizzle(
  async (sql, params, method) => {
    const sqlite = await getDb();
    let rows: any = [];
    let results = [];
    // If the query is a SELECT, use the select method
    if (isSelectQuery(sql)) {
      rows = await sqlite
        .select(sql, params)
        .catch((e) => {
          console.error("SQL Error:", e);
          throw e;
        }).finally(() => console.warn("SQL warn:", sql, params));
    } else {
      // Otherwise, use the execute method
      rows = await sqlite
        .execute(sql, params)
        .catch((e) => {
          console.error("SQL Error:", e);
          throw e;
        })
        .finally(() => console.warn("SQL warn:", sql, params));
      return { rows: [] };
    }

    rows = rows.map((row: any) => {
      return Object.values(row);
    });

    // If the method is "all", return all rows
    results = method === "all" ? rows : rows[0];
    return { rows: results };
  },
  // Pass the schema to the drizzle instance
  { relations: relations },
);

/**
 * Checks if the given SQL query is a SELECT query.
 * @param sql The SQL query to check.
 * @returns True if the query is a SELECT query, false otherwise.
 */
function isSelectQuery(sql: string): boolean {
  const selectRegex = /^\s*SELECT\b/i;
  return selectRegex.test(sql);
}
