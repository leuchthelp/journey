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
      API.Provider.password_auth(url, type, uname, psw),
      wrapWithEffect,
    );

    return yield* Effect.matchEffect(wrapped, {
      onFailure: (err) => {
        console.error(err);
        return Effect.succeed([undefined, url, uname, psw]);
      },
      onSuccess: (value) => Effect.succeed([value, url, "", ""]),
    }) as Effect.Effect<
      [ProviderKey | undefined, string, string, string],
      never,
      never
    >;
  });

const logOutOfProvider = (key?: ProviderKey) =>
  Effect.gen(function* () {
    const checkedKey = yield* pipe(key, Effect.fromNullishOr, guaranteeNoError);
    const wrapped = pipe(checkedKey, API.Provider.deregister, wrapWithEffect);

    return yield* Effect.matchEffect(wrapped, {
      onFailure: (err) => {
        console.error(err);
        return Effect.succeed(key);
      },
      onSuccess: () => Effect.succeed(undefined),
    });
  });

export { passwordAuth, logOutOfProvider };
