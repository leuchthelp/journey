export const ssr = false;

import { db } from "$lib/db/database";
import { mediaItems, providerItems } from "$lib/db/schema/schema";
import { providerDataQuery } from "$lib/db/queries";
import type { LayoutLoad } from "./$types";

export const load: LayoutLoad = async () => {
  //await db.delete(providerItems)
  // await db.select().from(mediaItems)
  // await db.delete(mediaItems)
  return {
    post: await providerDataQuery.execute(),
  };
};
