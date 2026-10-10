import type { LayoutLoad } from "./$types";
import { Effect } from "effect";
import { getProviders, getSupportedProviderTypes } from "#lib/effects/provider.ts";

export const ssr = false;

export const load: LayoutLoad = () => {
  return {
    providerReq: Effect.runPromise(getProviders()),
    supportedTypeReq: Effect.runPromise(getSupportedProviderTypes()),
  };
};
