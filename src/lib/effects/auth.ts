import type { ProviderVariant } from "../bindings.ts";
import { API } from "../proxy.ts";
import { Effect } from "effect";

const passwordAuth = (
  url: string,
  type: ProviderVariant,
  uname: string,
  psw: string,
) => {
  return Effect.gen(function* () {
    const apiResponse = yield* Effect.promise(
      async () => await API.provider.password_auth(url, type, uname, psw),
    );

    if (apiResponse.status === "error") yield* Effect.fail(apiResponse.error);
    else return yield* Effect.succeed(apiResponse.data);
  });
};

export { passwordAuth };
