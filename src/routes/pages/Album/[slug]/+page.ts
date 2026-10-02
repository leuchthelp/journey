import type { PageLoad } from "./$types";
import { AlbumItem } from "#lib/components/MediaItems/MediaItems.ts";

export const load: PageLoad = async () => {
  return {
    post: new AlbumItem(),
  };
};
