import type { ProviderVariant, ProviderKey } from "../bindings.ts";
import { API } from "../proxy.ts";
import { Effect, pipe } from "effect";
import { wrapWithEffect, checkIfUndefined } from "./generic.ts";

const passwordAuth = (
  url: string,
  type: ProviderVariant,
  uname: string,
  psw: string,
) => {
  return Effect.gen(function* () {
    const wrapped = wrapWithEffect(
      API.provider.password_auth(url, type, uname, psw),
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
};

const logOutOfProvider = (key: ProviderKey | undefined) => {
  return Effect.matchEffect(checkIfUndefined(key), {
    onFailure: (err) => {
      console.log(err);
      return Effect.succeed(key);
    },
    onSuccess: (value) => {
      const wrapped = pipe(value, API.provider.deregister, wrapWithEffect);

      return Effect.matchEffect(wrapped, {
        onFailure: (err) => {
          console.error(err);
          return Effect.succeed(value);
        },
        onSuccess: () => {
          return Effect.succeed(undefined);
        },
      });
    },
  });
};

export { passwordAuth, logOutOfProvider };
