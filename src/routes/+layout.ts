import { Effect } from "effect";

import { getProviders, getSupportedProviderTypes } from "#lib/effects/provider.ts";

import type { LayoutLoad } from "./$types";

export const ssr = false;

export const load: LayoutLoad = () => {
  return {
    providerReq: Effect.runPromise(getProviders()),
    supportedTypeReq: Effect.runPromise(getSupportedProviderTypes()),
  };
};
