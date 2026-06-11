import type { PageLoad } from "./$types.d.ts";
import { homeCache } from "$lib/components/MediaItems/ItemCache.ts";
import type { MediaItem } from "$lib/db/relations.ts";
import { mainPageDataQuery } from "$lib/db/queries.ts";

function toArrayClean<X>(xs: Iterable<X | undefined>): X[] {
  const res: X[] = [];

  for (const entry of xs) {
    if (entry) res.push(entry);
  }

  return res;
}

export const load: PageLoad = async () => {
  let res: MediaItem[];

  // Medium: look in cache if item has been posted already
  if (homeCache) {
    const tmp = homeCache.rvalues();
    res = toArrayClean(tmp);

    if (res.length !== 0) {
      return {
        post: res,
      };
    }
  }

  res = await mainPageDataQuery.execute({ limit: 6 });

  console.log(res);
  return { post: res };
};
