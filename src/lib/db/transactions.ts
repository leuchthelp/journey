import { db } from "./database";
import type { MediaItem } from "./relations";
import {
  imageItems,
  mediaItemChildren,
  mediaItems,
  mediaItemToImageItem,
  mediaItemToProviderItem,
} from "./schema/schema";

import { SQLiteTransaction } from "drizzle-orm/sqlite-core";
import type { SqliteRemoteResult } from "drizzle-orm/sqlite-proxy";
import { relations } from "./relations.ts";

type TransactionType = SQLiteTransaction<
  "async",
  SqliteRemoteResult,
  Record<string, never>,
  typeof relations
>;

export const insertMediaItem = async (item: MediaItem) => {
  await db.transaction(async (tx) => {
    await tx.insert(mediaItems).values(item);

    const newItemId = await tx.query.mediaItems.findFirst({
      where: { uuid: item.uuid },
      columns: { id: true },
    });

    console.log(newItemId);
    if (newItemId) {
      // Add reference to parents from child
      // Will also tell the parent which children it has
      // via the junction table
      for (const parent of item.parents) {
        const parentId = await tx.query.mediaItems.findFirst({
          where: { uuid: parent.uuid },
          columns: { id: true },
        });

        if (parentId)
          await tx.insert(mediaItemChildren).values({
            childId: newItemId.id,
            parentId: parentId.id,
          });
      }

      // Add reference to provider junction table
      // of which providers the Item can be got from.
      // Since providers are added to the DB before indexing
      // we just have to search for the required provider &
      // reference their Id.
      await tx.transaction(async (tx2) => {
        await providerTransaction(tx2, item, newItemId.id);
      });

      // Add add images and reference to image junction table.
      // Also need to add providers for image in a additional step
      // as a separate transaction.
      await tx.transaction(async (tx2) => {
        await imageTransaction(tx2, item, newItemId.id);
      });
    }
  });
};

const providerTransaction = async (
  tx: TransactionType,
  item: MediaItem,
  newItemId: number,
) => {
  for (const provider of item.providers) {
    const providerId = await tx.query.providerItems.findFirst({
      where: { serverId: provider.serverId },
      columns: { id: true },
    });
    if (providerId)
      await tx.insert(mediaItemToProviderItem).values({
        mediaItemId: newItemId,
        providerItemId: providerId.id,
      });
  }
};

const imageTransaction = async (
  tx: TransactionType,
  item: MediaItem,
  newItemId: number,
) => {
  for (const image of item.images) {
    await tx.insert(imageItems).values(image);

    const imageId = await tx.query.imageItems.findFirst({
      where: { url: image.url },
      columns: { id: true },
    });

    if (imageId)
      await tx.insert(mediaItemToImageItem).values({
        mediaItemId: newItemId,
        imageItemId: imageId.id,
      });
  }
};
