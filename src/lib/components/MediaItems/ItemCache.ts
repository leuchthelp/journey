import { LRUCache } from "lru-cache";
import { type MediaItemDTO } from "../../bindings.ts";

const options = {
  max: 150,

  // how long to live in ms
  ttl: 1000 * 60 * 5,

  // return stale items before removing from cache?
  allowStale: false,
  updateAgeOnGet: false,
  updateAgeOnHas: false,
};

const itemCache = new LRUCache<string, MediaItemDTO>(options);

export { itemCache };
