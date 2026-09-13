import { Effect } from "effect";
import type { TauRpcResult } from "../bindings.ts";

const wrapWithEffect = <T, E>(promise: Promise<TauRpcResult<T, E>>) => {
  return Effect.gen(function* () {
    const result = yield* Effect.tryPromise(async () => await promise);

    if (result.status === "error") yield* Effect.fail(result.error);
    else return yield* Effect.succeed(result.data);
  });
};

const guaranteeNoError = <T, E>(effect: Effect.Effect<T, E, never>) => {
  return Effect.matchEffect(effect, {
    onFailure: (err) => {
      /* 
      This needs to call a custom Error handler at some point in the future.
      The handler needs to throw up a notification telling the user a fatal
      failure has occurred. 
      It then needs to call the rust ErrorAPI passing every available detail
      & creating a log file entry. 

      Once that succeeds the current function gets killed.
      */
      console.error(err);
      alert(err);
      return Effect.die(effect);
    },
    onSuccess: (value) => {
      return Effect.succeed(value);
    },
  });
};

export { wrapWithEffect, guaranteeNoError };
