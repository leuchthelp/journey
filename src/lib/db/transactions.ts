import { db } from "./database";
import type { MediaItem } from "./relations";
import {
  contentItems,
  imageItems,
  mediaItemChildren,
  mediaItems,
  mediaItemToImageItem,
  mediaItemToProviderItem,
  originalItems,
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
    await tx.insert(mediaItems).values(item).onConflictDoNothing();

    const newItemId = await tx.query.mediaItems.findFirst({
      where: { uuid: item.uuid },
      columns: { uuid: true },
    });

    if (newItemId) {
      if (item.original.length !== 0)
        await tx.transaction(async (tx2) => {
          await originalTransaction(tx2, item);
        });

      if (item.content.length !== 0)
        await tx.transaction(async (tx2) => {
          await contentTransaction(tx2, item);
        });

      // Add reference to parents from child
      // Will also tell the parent which children it has
      // via the junction table
      if (item.parents.length !== 0)
        await tx.transaction(async (tx2) => {
          await parentTransaction(tx2, item, newItemId.uuid);
        });

      // Add reference to provider junction table
      // of which providers the Item can be got from.
      // Since providers are added to the DB before indexing
      // we just have to search for the required provider &
      // reference their Id.
      if (item.providers.length !== 0)
        await tx.transaction(async (tx2) => {
          await providerTransaction(tx2, item, newItemId.uuid);
        });

      // Add add images and reference to image junction table.
      // Also need to add providers for image in a additional step
      // as a separate transaction.
      if (item.images.length !== 0)
        await tx.transaction(async (tx2) => {
          await imageTransaction(tx2, item, newItemId.uuid);
        });
    }
  });
};

const originalTransaction = async (tx: TransactionType, item: MediaItem) => {
  for (const original of item.original) {
    const exists = await tx.query.originalItems.findFirst({
      where: {
        uuid: original.uuid,
        parentId: original.parentId,
        serverId: original.serverId,
      },
    });

    if (exists === undefined)
      await tx.insert(originalItems).values(original).onConflictDoNothing();
  }
};

const contentTransaction = async (tx: TransactionType, item: MediaItem) => {
  for (const content of item.content) {
    const exists = await tx.query.contentItems.findFirst({
      where: {
        description: content.description,
        type: content.type,
        parentId: content.parentId,
      },
    });

    if (exists === undefined)
      await tx.insert(contentItems).values(content).onConflictDoNothing();
  }
};

const parentTransaction = async (
  tx: TransactionType,
  item: MediaItem,
  newItemId: string,
) => {
  for (const parent of item.parents) {
    const parentId = await tx.query.mediaItems.findFirst({
      where: { uuid: parent.uuid },
      columns: { uuid: true },
    });

    if (parentId)
      await tx
        .insert(mediaItemChildren)
        .values({
          childId: newItemId,
          parentId: parentId.uuid,
        })
        .onConflictDoNothing();
  }
};

const providerTransaction = async (
  tx: TransactionType,
  item: MediaItem,
  newItemId: string,
) => {
  for (const provider of item.providers) {
    const providerId = await tx.query.providerItems.findFirst({
      where: { serverId: provider.serverId },
      columns: { userId: true },
    });
    if (providerId)
      await tx
        .insert(mediaItemToProviderItem)
        .values({
          mediaItemId: newItemId,
          providerItemId: providerId.userId,
        })
        .onConflictDoNothing();
  }
};

const imageTransaction = async (
  tx: TransactionType,
  item: MediaItem,
  newItemId: string,
) => {
  for (const image of item.images) {
    await tx.insert(imageItems).values(image).onConflictDoNothing();

    const imageId = await tx.query.imageItems.findFirst({
      where: { url: image.url },
      columns: { url: true },
    });

    if (imageId)
      await tx
        .insert(mediaItemToImageItem)
        .values({
          mediaItemId: newItemId,
          imageItemId: imageId.url,
        })
        .onConflictDoNothing();
  }
};
