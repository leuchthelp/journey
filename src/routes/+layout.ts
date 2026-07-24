export const ssr = false;
import { type ProviderDTO } from "#lib/bindings.ts";

import type { LayoutLoad } from "./$types.d.ts";

export const load: LayoutLoad = async () => {
  //indexedDB.deleteDatabase("/pglite/dev");
  const tmp: ProviderDTO[] = [];
  return {
    post: tmp,
  };
};
