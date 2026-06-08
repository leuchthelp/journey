import * as schema from "./schema/schema";
import { defineRelations, type BuildQueryResult } from "drizzle-orm";

export const relations = defineRelations(schema, (r) => ({
  mediaItems: {
    original: r.many.originalItems({
      from: r.mediaItems.uuid,
      to: r.originalItems.parentId,
    }),
    content: r.many.contentItems({
      from: r.mediaItems.uuid,
      to: r.contentItems.parentId,
    }),
    providers: r.many.providerItems({
      from: r.mediaItems.uuid.through(r.mediaItemToProviderItem.mediaItemId),
      to: r.providerItems.userId.through(
        r.mediaItemToProviderItem.providerItemId,
      ),
    }),
    images: r.many.imageItems({
      from: r.mediaItems.uuid.through(r.mediaItemToImageItem.mediaItemId),
      to: r.imageItems.url.through(r.mediaItemToImageItem.imageItemId),
    }),
    children: r.many.mediaItems({
      from: r.mediaItems.uuid.through(r.mediaItemChildren.parentId),
      to: r.mediaItems.uuid.through(r.mediaItemChildren.childId),
    }),
    parents: r.many.mediaItems({
      from: r.mediaItems.uuid.through(r.mediaItemChildren.childId),
      to: r.mediaItems.uuid.through(r.mediaItemChildren.parentId),
    }),
  },

  providerItems: {
    mediaItems: r.many.mediaItems(),
    imageItems: r.many.imageItems({
      from: r.providerItems.serverId,
      to: r.imageItems.serverId,
    }),
  },

  imageItems: {
    mediaItems: r.many.mediaItems(),
  },
}));

type Relations = typeof relations;

export type MediaItem = BuildQueryResult<
  Relations,
  Relations["mediaItems"],
  {
    with: {
      original: { columns: { id: false } };
      content: { columns: { id: false } };
      providers: true;
      images: true;
      parents: { columns: { uuid: true } };
    };
  }
>;

export type ParentItem = BuildQueryResult<
  Relations,
  Relations["mediaItems"],
  {
    columns: { uuid: true };
  }
>;
