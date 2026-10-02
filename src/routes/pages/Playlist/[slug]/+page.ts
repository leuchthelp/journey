import type { PageLoad } from "./$types";
import { PlaylistItem } from "#lib/components/MediaItems/MediaItems.ts";

export const load: PageLoad = async () => {
  return {
    post: new PlaylistItem(),
  };
};
