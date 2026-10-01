import type { PageLoad } from "./$types.d.ts";
import { PlaylistItem } from "#lib/components/MediaItems/MediaItems.ts";

export const load: PageLoad = async () => {
  return {
    post: new PlaylistItem(),
  };
};
