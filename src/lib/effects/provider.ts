import { Effect, pipe } from "effect";
import { guaranteeNoError, wrapWithEffect } from "#lib/effects/generic.ts";
import type {
  IndexerKey,
  IndexerMsg,
  ProviderDTO,
  ProviderKey,
  ProviderAuthSchema,
  ProviderVariant,
} from "../bindings.ts";
import { API } from "../proxy.ts";

const getSupportedProviderVariants = () =>
  Effect.gen(function* () {
    const wrapped = pipe(API.Provider.get_supported_variants, Effect.promise);

    return yield* Effect.matchEffect(wrapped, {
      onFailure: (err) => {
        console.error(err);
        return Effect.succeed([] as ProviderVariant[]);
      },
      onSuccess: (value) => Effect.succeed(value),
    });
  });

const getSupportedAuthSchema = (variant: ProviderVariant) =>
  Effect.gen(function* () {
    const wrapped = pipe(
      API.Provider.get_supported_auth_schema(variant),
      wrapWithEffect,
    );

    return yield* Effect.matchEffect(wrapped, {
      onFailure: (err) => {
        console.error(err);
        return Effect.succeed([] as ProviderAuthSchema[]);
      },
      onSuccess: (value) => Effect.succeed(value),
    });
  });

const getProvider = (key?: ProviderKey) =>
  Effect.matchEffect(pipe(key, Effect.fromNullishOr), {
    onFailure: () => {
      console.warn("No known provider yet, offering to create new one.");
      return Effect.succeed(undefined);
    },
    onSuccess: (checkedKey) => {
      const wrapped = pipe(
        checkedKey,
        API.Provider.get_provider,
        wrapWithEffect,
      );

      return Effect.matchEffect(wrapped, {
        onFailure: (err) => {
          console.error(err);
          return Effect.succeed(undefined);
        },
        onSuccess: (value) => Effect.succeed(value),
      });
    },
  });

const getProviders = () =>
  Effect.gen(function* () {
    const wrapped = pipe(API.Provider.get_providers(), wrapWithEffect);

    return yield* Effect.matchEffect(wrapped, {
      onFailure: (err) => {
        console.error(err);
        return Effect.succeed([] as ProviderDTO[]);
      },
      onSuccess: (value) => Effect.succeed(value),
    });
  });

const setIndexerKey = (provider?: ProviderDTO) =>
  Effect.matchEffect(pipe(provider, Effect.fromNullishOr), {
    onFailure: () => {
      console.warn("No known provider yet, therefore no indexer.");
      return Effect.succeed(undefined);
    },
    onSuccess: (checkedProvider) => {
      const key: IndexerKey = {
        providerId: checkedProvider.key.providerId,
        variant: checkedProvider.type,
      };
      return Effect.succeed(key);
    },
  });

const indexerStatus = (
  callback: (response: IndexerMsg) => void,
  key?: IndexerKey,
) =>
  Effect.gen(function* () {
    const checkedKey = yield* pipe(key, Effect.fromNullishOr, guaranteeNoError);

    const wrapped = pipe(
      API.Provider.indexer_status(checkedKey, callback),
      wrapWithEffect,
    );

    return yield* Effect.matchEffect(wrapped, {
      onFailure: (err) => {
        console.error(err);
        return Effect.succeed(false);
      },
      onSuccess: () => Effect.succeed(true),
    });
  });

export {
  getSupportedProviderVariants,
  getSupportedAuthSchema,
  getProvider,
  getProviders,
  setIndexerKey,
  indexerStatus,
};
