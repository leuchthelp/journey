export const ssr = false;

import { db } from "$lib/db/database.ts";
import * as schema from "$lib/db/schema/schema.ts";
import type { LayoutLoad } from "./$types";

export const load: LayoutLoad = async () => {
  await db.delete(schema.providerItems)
  return { post: await db.select().from(schema.providerItems) };
};
