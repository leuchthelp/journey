import { error } from "@sveltejs/kit";
import type { PageLoad } from "./$types";
import { page } from "$app/state";
import { itemCache } from "$lib/components/MediaItems/ItemCache.ts";
import { GenreItem } from "$lib/components/MediaItems/MediaItems";
import { singlePageDataQuery } from "$lib/db/queries";

export const load: PageLoad = async ({ params }) => {
  // Fastest: try check out parent page if it already posted the item
  let data = page.data.post as GenreItem[];
  let res: GenreItem | undefined;
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
        post: [tmp],
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
