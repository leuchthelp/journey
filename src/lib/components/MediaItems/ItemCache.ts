import { LRUCache } from "lru-cache";
import type { MediaItems } from "../../db/schema/schema.ts";

const options = {
  max: 150,

  // how long to live in ms
  ttl: 1000 * 60 * 5,

  // return stale items before removing from cache?
  allowStale: false,
  updateAgeOnGet: false,
  updateAgeOnHas: false,
};

export const itemCache = new LRUCache<string, MediaItems>(options);
export const homeCache = new LRUCache<string, MediaItems>(options);
