export const ssr = false;

import { db } from "$lib/db/database";
import { providerItems } from "$lib/db/schema/schema";
import { providerDataQuery } from "$lib/db/queries";
import type { LayoutLoad } from "./$types";

export const load: LayoutLoad = async () => {
  //await db.delete(providerItems)
  return {
    post: await providerDataQuery.execute(),
  };
};
