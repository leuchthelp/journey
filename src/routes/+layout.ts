export const ssr = false;

import { providerDataQuery } from "$lib/db/queries.ts";
import type { LayoutLoad } from "./$types.d.ts";
import { migrate_pglite } from "../lib/db/migrate.ts";

export const load: LayoutLoad = async () => {
  migrate_pglite().catch((e) => {
    console.error("migration success", e);
  });
  return {
    post: await providerDataQuery.execute(),
  };
};
