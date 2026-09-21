import { type MediaItemDTO } from "#lib/bindings.ts";
import type { PageLoad } from "./$types";

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
    providerReq: tmp,
  };
};
