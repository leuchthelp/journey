import * as schema from "./schema/schema";
import { defineRelations } from "drizzle-orm";

export const relations = defineRelations(schema, (r) => ({
  mediaItems: {
    content: r.many.contentItems({
      from: r.mediaItems.id,
      to: r.contentItems.parentId,
    }),
    providers: r.many.providerItems({
      from: r.mediaItems.id.through(r.mediaItemToProviderItem.mediaItemId),
      to: r.providerItems.id.through(r.mediaItemToProviderItem.providerItemId),
    }),
    imageItems: r.many.imageItems({
      from: r.mediaItems.id.through(r.mediaItemToImageItem.mediaItemId),
      to: r.imageItems.id.through(r.mediaItemToImageItem.imageItemId),
    }),
    children: r.many.mediaItems({
      from: r.mediaItems.id.through(r.mediaItemChildren.parentId),
      to: r.mediaItems.id.through(r.mediaItemChildren.childId),
    }),
    parent: r.many.mediaItems({
      from: r.mediaItems.id.through(r.mediaItemChildren.childId),
      to: r.mediaItems.id.through(r.mediaItemChildren.parentId),
    }),
  },

  providerItems: {
    mediaItems: r.many.mediaItems(),
    imageItemItems: r.many.imageItems(),
  },

  imageItems: {
    mediaItems: r.many.mediaItems(),
    imageItem: r.one.providerItems({
      from: r.imageItems.providerId,
      to: r.providerItems.id,
    }),
  },
}));
