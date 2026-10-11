import { Effect, pipe } from "effect";

import { API } from "../proxy.ts";
import { wrapWithEffect } from "./generic.ts";

const play = (immediately: boolean) =>
  Effect.matchEffect(pipe(immediately, API.Player.play, wrapWithEffect), {
    onFailure: (err) => {
      console.error(err);
      return Effect.succeed(undefined);
    },
    onSuccess: (duration) => Effect.succeed(duration),
  });

const pause = () =>
  Effect.matchEffect(pipe(API.Player.pause(), wrapWithEffect), {
    onFailure: (err) => {
      console.error(err);
      return Effect.succeed(undefined);
    },
    onSuccess: (duration) => Effect.succeed(duration),
  });

export { play, pause };
