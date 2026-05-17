import { db } from "../db/database";
import * as schema from "../db/schema";
import type { PageLoad } from "./$types";

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

export const load: PageLoad = async () => {
  return { post: await db.query.mediaItems.findMany().execute() };
};
