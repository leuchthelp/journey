import { Effect, pipe } from "effect";

import type { MediaItemDTO, MediaItemType } from "../bindings.ts";
import { API } from "../proxy.ts";
import { wrapWithEffect } from "./generic.ts";

const getMediaItems = (type: MediaItemType, amount: number = 6) =>
  Effect.matchEffect(pipe(API.MediaItem.get_media_items(type, amount), wrapWithEffect), {
    onFailure: (err) => {
      console.error(err);
      return Effect.succeed([] as MediaItemDTO[]);
    },
    onSuccess: (value) => Effect.succeed(value),
  });

const getMediaItem = (type: MediaItemType, uuid: string) =>
  Effect.matchEffect(pipe(API.MediaItem.get_media_item(type, uuid), wrapWithEffect), {
    onFailure: (err) => {
      console.error(err);
      return Effect.succeed(undefined);
    },
    onSuccess: (value) => Effect.succeed(value),
  });

export { getMediaItems, getMediaItem };
