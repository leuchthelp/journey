import { PlaylistItem } from "#lib/components/MediaItems/MediaItems.ts";

import type { PageLoad } from "./$types";

export const load: PageLoad = async () => {
  return {
    post: new PlaylistItem(),
  };
};
