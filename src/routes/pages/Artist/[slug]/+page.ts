import type { PageLoad } from "./$types.d.ts";

import { ArtistItem } from "#lib/components/MediaItems/MediaItems.ts";

export const load: PageLoad = async () => {
  return {
    post: new ArtistItem(),
  };
};
