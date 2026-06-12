export const ssr = false;

import { providerDataQuery } from "$lib/db/queries.ts";
import type { LayoutLoad } from "./$types.d.ts";
import Database from "@tauri-apps/plugin-sql";
import { migrate_pglite } from "../lib/db/migrate.ts";

export const load: LayoutLoad = async () => {
  // let tmp = await Database.load("sqlite:dev.db")
  // await tmp.execute("DELETE FROM MediaItems")
  // await tmp.execute("DELETE FROM ProviderItems")

  migrate_pglite().then(() => {
    console.log("migration success");
  });

  const res = await providerDataQuery.execute();
  return {
    post: res,
  };
};
