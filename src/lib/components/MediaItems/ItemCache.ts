import { LRUCache } from "lru-cache";
import { type MediaItems } from "$lib/db/schema";

const options = {
  max: 25,

  // how long to live in ms
  ttl: 1000 * 60 * 5,

  // return stale items before removing from cache?
  allowStale: false,
  updateAgeOnGet: false,
  updateAgeOnHas: false,
};

export const itemCache = new LRUCache<string, MediaItems>(options);
