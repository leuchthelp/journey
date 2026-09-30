import { Effect } from "effect";
import { getMediaItems } from "../lib/effects/media_item.ts";
import type { PageLoad } from "./$types";

export const load: PageLoad = async ({ parent, depends }) => {
  await parent();

  depends("app:mainPage");

  return {
    mediaItemsReq: Effect.runPromise(getMediaItems("Audio")),
  };
};
