import PasswordAuth from "./PasswordAuth.svelte";
import { type ProviderAuthSchema } from "../../../bindings.ts";
import { type LegacyComponentType } from "svelte/legacy";

export const supportedAuthSchema = new Map<
  ProviderAuthSchema,
  LegacyComponentType
>([["Password", PasswordAuth]]);
