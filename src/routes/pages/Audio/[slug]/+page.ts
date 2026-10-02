import type { PageLoad } from "./$types";
import { Effect } from "effect";
import { getMediaItem } from "#lib/effects/media_item.ts";

export const load: PageLoad = async ({ params }) => {
  return {
    item: await Effect.runPromise(getMediaItem("Audio", params.slug)),
  };
};
