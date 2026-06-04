import { db } from "$lib/db/database.ts";
import type { PageLoad } from "./$types";
import { homeCache } from "$lib/components/MediaItems/ItemCache.ts";
import type { MediaItem } from "$lib/db/relations";

function toArrayClean<X>(xs: Iterable<X | undefined>): X[] {
  let res: X[] = [];

  for (let entry of xs) {
    if (entry) res.push(entry);
  }

  return res;
}

export const load: PageLoad = async () => {
  let res: MediaItem[];

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

  res = await db.query.mediaItems.findMany({
    limit: 6,
    columns: { id: false },
    with: {
      content: { columns: { id: false, parentId: false } },
      providers: { columns: { id: false } },
      images: { columns: { id: false, providerId: false } },
    },
  });
  return { post: res };
};
