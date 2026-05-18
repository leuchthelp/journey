import { db } from "$lib/db/database";
import * as schema from "$lib/db/schema";
import type { PageLoad } from "./$types";
import { homeCache } from "$lib/components/MediaItems/ItemCache";

// import {
//   ArtistItem,
//   GenreItem,
//   PlaylistItem,
//   SongItem,
// } from "$lib/components/MediaItems/MediaItems";

// const test = new SongItem();
// const test2 = new ArtistItem();
// const test3 = new GenreItem();
// const test4 = new PlaylistItem();

// test.hash = "SongItem";
// test2.hash = "ArtistItem";
// test3.hash = "GenreItem";
// test4.hash = "PlaylistItem";

// test.content = "SongItem";
// test2.content = "ArtistItem";
// test3.content = "GenreItem";
// test4.content = "PlaylistItem";
// await db.insert(schema.mediaItems).values([test, test2, test3, test4])
//await db.delete(schema.mediaItems)
function toArrayClean<X>(xs: Iterable<X | undefined>): X[] {
  let res: X[] = [];

  for (let entry of xs) {
    if (entry) res.push(entry);
  }

  return res;
}

export const load: PageLoad = async () => {
  let res: schema.MediaItems[];

  // Medium: look in cache if item has been posted already
  if (homeCache) {
    let tmp = homeCache.rvalues();
    res = toArrayClean(tmp);

    if (res.length !== 0)
      return {
        post: res,
      };
  }

  res = await db.select().from(schema.mediaItems).limit(6);
  return { post: res };
};
