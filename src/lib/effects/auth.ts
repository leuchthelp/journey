import type { ProviderVariant, ProviderKey } from "../bindings.ts";
import { API } from "../proxy.ts";
import { Effect, pipe } from "effect";
import { wrapWithEffect, guaranteeNoError } from "./generic.ts";

const passwordAuth = (
  url: string,
  type: ProviderVariant,
  uname: string,
  psw: string,
) =>
  Effect.gen(function* () {
    const wrapped = pipe(
      API.provider.password_auth(url, type, uname, psw),
      wrapWithEffect,
    );

    return yield* Effect.matchEffect(wrapped, {
      onFailure: (err) => {
        console.error(err);
        return Effect.succeed([undefined, uname, psw]);
      },
      onSuccess: (value) => {
        return Effect.succeed([value, "", ""]);
      },
    }) as Effect.Effect<
      [ProviderKey | undefined, string, string],
      never,
      never
    >;
  });

const logOutOfProvider = (key?: ProviderKey) =>
  Effect.gen(function* () {
    const checkedKey = yield* pipe(key, Effect.fromNullishOr, guaranteeNoError);
    const wrapped = pipe(checkedKey, API.provider.deregister, wrapWithEffect);

    return yield* Effect.matchEffect(wrapped, {
      onFailure: (err) => {
        console.error(err);
        return Effect.succeed(key);
      },
      onSuccess: () => {
        return Effect.succeed(undefined);
      },
    });
  });

export { passwordAuth, logOutOfProvider };
