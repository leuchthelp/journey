export const ssr = false;
import { type ProviderDTO } from "#lib/bindings.ts";
import { API } from "#lib/proxy.ts";
import type { LayoutLoad } from "./$types.d.ts";

export const load: LayoutLoad = async () => {
  //indexedDB.deleteDatabase("/pglite/dev");

  await API.provider.get_providers((provider) => {
    console.log(provider);
  });

  const tmp: ProviderDTO[] = [];
  return {
    post: tmp,
  };
};
