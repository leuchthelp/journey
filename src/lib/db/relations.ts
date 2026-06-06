import * as schema from "./schema/schema";
import { defineRelations, type BuildQueryResult } from "drizzle-orm";

export const relations = defineRelations(schema, (r) => ({
  mediaItems: {
    content: r.many.contentItems(),
    providers: r.many.providerItems({
      from: r.mediaItems.id.through(r.mediaItemToProviderItem.mediaItemId),
      to: r.providerItems.id.through(r.mediaItemToProviderItem.providerItemId),
    }),
    images: r.many.imageItems({
      from: r.mediaItems.id.through(r.mediaItemToImageItem.mediaItemId),
      to: r.imageItems.id.through(r.mediaItemToImageItem.imageItemId),
    }),
    children: r.many.mediaItems({
      from: r.mediaItems.id.through(r.mediaItemChildren.parentId),
      to: r.mediaItems.id.through(r.mediaItemChildren.childId),
    }),
    parents: r.many.mediaItems({
      from: r.mediaItems.id.through(r.mediaItemChildren.childId),
      to: r.mediaItems.id.through(r.mediaItemChildren.parentId),
    }),
  },

  contentItems: {
    parent: r.one.mediaItems({
      from: r.contentItems.parentId,
      to: r.mediaItems.id,
    }),
  },

  providerItems: {
    mediaItems: r.many.mediaItems(),
    imageItems: r.many.imageItems(),
  },

  imageItems: {
    mediaItems: r.many.mediaItems(),
    image: r.one.providerItems({
      from: r.imageItems.serverId,
      to: r.providerItems.id,
    }),
  },
}));

type Relations = typeof relations;

export type MediaItem = BuildQueryResult<
  Relations,
  Relations["mediaItems"],
  {
    columns: { id: false };
    with: {
      content: { columns: { id: false; parentId: false } };
      providers: { columns: { id: false } };
      images: { columns: { id: false; providerId: false } };
      parents: { columns: { id: false } };
    };
  }
>;
