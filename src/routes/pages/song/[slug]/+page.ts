import { db } from "$lib/db/database";
import { error } from "@sveltejs/kit";
import type { PageLoad } from "./$types";
import * as schema from "$lib/db/schema";
import { SongItem } from "$lib/components/MediaItems/MediaItems";
import { eq } from "drizzle-orm";
import { page } from "$app/state";
import { itemCache } from "$lib/components/MediaItems/ItemCache";

export const load: PageLoad = async ({ params }) => {
  // Fastest: try check out parent page if it already posted the item
  let data = page.data.post as SongItem[];
  let res: SongItem | undefined;
  if (data) {
    let res = data.filter((item) => item.hash === params.slug)[0];

    if (res)
      return {
        post: res,
      };
  }

  // Medium: look in cache if item has been posted already
  if (itemCache) {
    res = itemCache.get(params.slug);
    if (res)
      return {
        post: res,
      };
  }

  // Brutal: fallback to database to get item fresh
  let tmp = await db
    .select()
    .from(schema.mediaItems)
    .where(eq(schema.mediaItems.hash, params.slug))
    .limit(1);

  res = tmp[0];
  if (res)
    return {
      post: res,
    };

  error(404, "Not found");
};
