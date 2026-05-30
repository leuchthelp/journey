import { db } from "$lib/db/database.ts";
import * as schema from "$lib/db/schema/schema.ts";
import type { PageLoad } from "./$types";
import { homeCache } from "$lib/components/MediaItems/ItemCache.ts";

function toArrayClean<X>(xs: Iterable<X | undefined>): X[] {
  let res: X[] = [];

  for (let entry of xs) {
    if (entry) res.push(entry);
  }

  return res;
}

export const load: PageLoad = async () => {
  let res: schema.MediaItems[];

  // Medium: look in cache if item has been posted already
  if (homeCache) {
    let tmp = homeCache.rvalues();
    res = toArrayClean(tmp);

    if (res.length !== 0) {
      return {
        post: res,
      };
    }
  }

  res = await db.select().from(schema.mediaItems).limit(6);
  return { post: res };
};
