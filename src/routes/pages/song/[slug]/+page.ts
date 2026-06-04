import { db } from "$lib/db/database.ts";
import { error } from "@sveltejs/kit";
import type { PageLoad } from "./$types";
import { page } from "$app/state";
import { itemCache } from "$lib/components/MediaItems/ItemCache.ts";
import { SongItem } from "$lib/components/MediaItems/MediaItems";

export const load: PageLoad = async ({ params }) => {
  // Fastest: try check out parent page if it already posted the item
  let data = page.data.post as SongItem[];
  let res: SongItem | undefined;
  if (data) {
    res = data.filter((item) => item.uuid === params.slug)[0];

    if (res) {
      return {
        post: res,
      };
    }
  }

  // Medium: look in cache if item has been posted already
  if (itemCache) {
    res = itemCache.get(params.slug);
    if (res) {
      return {
        post: res,
      };
    }
  }

  // Brutal: fallback to database to get item fresh
  res = await db.query.mediaItems.findFirst({
    where: { uuid: params.slug },
    columns: { id: false },
    with: {
      content: { columns: { id: false, parentId: false } },
      providers: { columns: { id: false } },
      images: { columns: { id: false, providerId: false } },
    },
  });

  if (res) {
    return {
      post: res,
    };
  }

  error(404, "Not found");
};
