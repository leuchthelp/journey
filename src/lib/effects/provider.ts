import { Effect, pipe } from "effect";
import { guaranteeNoError, wrapWithEffect } from "#lib/effects/generic.ts";
import type { IndexerKey, IndexerMsg, ProviderKey } from "../bindings.ts";
import { API } from "../proxy.ts";

const getProvider = (key?: ProviderKey) => {
  return Effect.matchEffect(pipe(key, Effect.fromNullishOr), {
    onFailure: () => {
      console.warn("No known provider yet, offering to create new one.");
      return Effect.succeed(undefined);
    },
    onSuccess: (checkedKey) => {
      const wrapped = pipe(
        checkedKey,
        API.provider.get_provider,
        wrapWithEffect,
      );

      return Effect.matchEffect(wrapped, {
        onFailure: (err) => {
          console.error(err);
          return Effect.succeed(undefined);
        },
        onSuccess: (value) => {
          return Effect.succeed(value);
        },
      });
    },
  });
};

const indexerStatus = (
  callback: (response: IndexerMsg) => void,
  key?: IndexerKey,
) => {
  return Effect.gen(function* () {
    const checkedKey = yield* pipe(key, Effect.fromNullishOr, guaranteeNoError);

    const wrapped = pipe(
      API.provider.indexer_status(checkedKey, callback),
      wrapWithEffect,
    );

    return yield* Effect.matchEffect(wrapped, {
      onFailure: (err) => {
        console.error(err);
        return Effect.succeed(false);
      },
      onSuccess: () => {
        return Effect.succeed(true);
      },
    });
  });
};

export { getProvider, indexerStatus };
