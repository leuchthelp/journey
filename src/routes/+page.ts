import type { PageLoad } from "./$types.d.ts";
import { type MediaItemDTO } from "#lib/bindings.ts";
// import { itemCache } from "$lib/components/MediaItems/ItemCache.ts";
// import type { MediaItem } from "$lib/db/relations.ts";

// function toArrayClean<X>(xs: Iterable<X | undefined>): X[] {
//   const res: X[] = [];

//   for (const entry of xs) {
//     if (entry) res.push(entry);
//   }

//   return res;
// }

export const load: PageLoad = async ({ parent, depends }) => {
  await parent();

  depends("app:mainPage");
  // let res: MediaItem[];
  // Medium: look in cache if item has been posted already
  // if (itemCache) {
  //   const tmp = itemCache.rvalues();
  //   res = toArrayClean(tmp);

  //   if (res.length !== 0) {
  //     return {
  //       post: res,
  //     };
  //   }
  // }
  const tmp: MediaItemDTO[] = [];
  return {
    post: tmp,
  };
};
