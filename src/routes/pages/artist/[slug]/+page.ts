import { error } from "@sveltejs/kit";
import type { PageLoad } from "./$types.d.ts";
import { page } from "$app/state";
import { itemCache } from "$lib/components/MediaItems/ItemCache.ts";
import { ArtistItem } from "$lib/components/MediaItems/MediaItems.ts";
import { singlePageDataQuery } from "$lib/db/queries.ts";

export const load: PageLoad = async ({ params }) => {
  // Fastest: try check out parent page if it already posted the item
  const data = page.data.post as ArtistItem[];
  let res: ArtistItem | undefined;
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
    let tmp = itemCache.get(params.slug);
    if (tmp) {
      return {
        post: tmp as ArtistItem,
      };
    }
  }

  // Brutal: fallback to database to get item fresh
  res = await singlePageDataQuery.execute({ slug: params.slug });

  if (res) {
    return {
      post: res,
    };
  }

  error(404, "Not found");
};
