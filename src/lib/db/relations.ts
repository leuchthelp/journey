import * as schema from "./schema/schema";
import { defineRelations } from "drizzle-orm";

export const relations = defineRelations(schema, (r) => ({
  mediaItems: {
    imageItems: r.many.imageItems({
      from: r.mediaItems.id,
      to: r.imageItems.id,
    }),
    provider: r.many.providerItems({
      from: r.mediaItems.id,
      to: r.providerItems.id,
    }),
    children: r.many.mediaItems({
      from: r.mediaItems.id,
      to: r.mediaItems.id,
    }),
  },
}));
