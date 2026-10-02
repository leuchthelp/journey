import type { PageLoad } from "./$types";

import { ArtistItem } from "#lib/components/MediaItems/MediaItems.ts";

export const load: PageLoad = async () => {
  return {
    post: new ArtistItem(),
  };
};
