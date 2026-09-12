import { Effect, UndefinedOr, Data } from "effect";
import type { TauRpcResult } from "../bindings.ts";

const wrapWithEffect = <T, E>(promise: Promise<TauRpcResult<T, E>>) => {
  return Effect.gen(function* () {
    const result = yield* Effect.promise(async () => await promise);

    if (result.status === "error") yield* Effect.fail(result.error);
    else return yield* Effect.succeed(result.data);
  }) as Effect.Effect<T, E, never>;
};

class EncounteredUndefinedError extends Data.TaggedError(
  "EncounteredUndefinedError",
)<{}> {}

const checkIfUndefined = <T>(value: T | undefined) => {
  return Effect.gen(function* () {
    return yield* UndefinedOr.match(value, {
      onUndefined: () => Effect.fail(new EncounteredUndefinedError()),
      onDefined: (value) => Effect.succeed(value),
    });
  });
};

export { wrapWithEffect, checkIfUndefined, EncounteredUndefinedError };
