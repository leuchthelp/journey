import { db } from "$lib/db/database";
import { error } from "@sveltejs/kit";
import type { PageLoad } from "./$types";
import * as schema from "$lib/db/schema";
import { eq } from "drizzle-orm";
import { page } from "$app/state";
import { itemCache } from "$lib/components/MediaItems/ItemCache";

export const load: PageLoad = async ({ params }) => {
  // Fastest: try check out parent page if it already posted the item
  let data = page.data.post as schema.MediaItems[];
  let res: schema.MediaItems[];
  if (data !== undefined) {
    res = data.filter((item) => item.hash === params.slug);

    if (res.length !== 0)
      return {
        post: res,
      };
  }

  // Medium: look in cache if item has been posted already
  if (itemCache !== undefined) {
    let tmp = itemCache.get(params.slug);
    if (tmp !== undefined)
      return {
        post: [tmp],
      };
  }

  // Brutal: fallback to database to get item fresh
  res = await db
    .select()
    .from(schema.mediaItems)
    .where(eq(schema.mediaItems.hash, params.slug))
    .limit(1);

  if (res.length !== 0)
    return {
      post: res,
    };

  error(404, "Not found");
};
