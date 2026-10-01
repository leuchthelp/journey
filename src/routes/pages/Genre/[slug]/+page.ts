import type { PageLoad } from "./$types.d.ts";
import { GenreItem } from "#lib/components/MediaItems/MediaItems.ts";

export const load: PageLoad = async () => {
  return {
    post: new GenreItem(),
  };
};
