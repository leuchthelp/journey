export const ssr = false;

import { db } from "$lib/db/database";
import type { LayoutLoad } from "./$types";

export const load: LayoutLoad = async () => {
  //await db.delete(schema.providerItems)
  return {
    post: await db.query.mediaItems.findMany({
      columns: { id: false },
      with: {
        content: { columns: { id: false, parentId: false } },
        providers: { columns: { id: false } },
        images: { columns: { id: false, providerId: false } },
      },
    }),
  };
};
