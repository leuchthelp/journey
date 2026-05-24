import { db } from "../lib/db/database.ts";
import * as schema from "../lib/db/schema.ts";
import type { PageLoad } from "../../.svelte-kit/types/src/routes/$types.d.ts";
import { homeCache } from "../lib/components/MediaItems/ItemCache.ts";
import { testJf, testJf2 } from "../lib/providers/JellyfinProvider.ts";

function toArrayClean<X>(xs: Iterable<X | undefined>): X[] {
  let res: X[] = [];

  for (let entry of xs) {
    if (entry) res.push(entry);
  }

  return res;
}

export const load: PageLoad = async () => {
  let res: schema.MediaItems[];

  let url = await testJf();
  console.log(url.config.url);

  let test2 = await testJf2();

  // Medium: look in cache if item has been posted already
  if (homeCache) {
    let tmp = homeCache.rvalues();
    res = toArrayClean(tmp);

    if (res.length !== 0)
      return {
        post: res,
        url: url,
        tmp: test2,
      };
  }

  res = await db.select().from(schema.mediaItems).limit(6);
  return { post: res, url: url, tmp: test2 };
};
