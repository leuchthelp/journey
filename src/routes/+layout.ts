export const ssr = false;

import { providerDataQuery } from "$lib/db/queries";
import type { LayoutLoad } from "./$types";
import Database from "@tauri-apps/plugin-sql";

export const load: LayoutLoad = async () => {
  // let tmp = await Database.load("sqlite:dev.db")
  // await tmp.execute("DELETE FROM MediaItems")
  // await tmp.execute("DELETE FROM ProviderItems")

  return {
    post: await providerDataQuery.execute(),
  };
};
