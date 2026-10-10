import { type LegacyComponentType } from "svelte/legacy";

import { type ProviderAuthSchema } from "../../../bindings.ts";
import PasswordAuth from "./PasswordAuth.svelte";

export const authSchemaComponents = new Map<ProviderAuthSchema, LegacyComponentType>([
  ["Password", PasswordAuth],
]);
