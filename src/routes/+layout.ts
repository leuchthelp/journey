import type { LayoutLoad } from "./$types";
import { Effect } from "effect";
import {
  getProviders,
  getSupportedProviderVariants,
} from "#lib/effects/provider.ts";

export const ssr = false;

export const load: LayoutLoad = () => {
  return {
    providerReq: Effect.runPromise(getProviders()),
    supportedVariantReq: Effect.runPromise(getSupportedProviderVariants()),
  };
};
