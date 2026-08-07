import JellyfinAuth from "./JellyfinAuth.svelte";
import { type ProviderVariant } from "#lib/bindings.ts";
import { type LegacyComponentType } from "svelte/legacy";

export const providerAuthOptions = new Map<
  ProviderVariant,
  LegacyComponentType
>([["JellyfinProvider", JellyfinAuth]]);
