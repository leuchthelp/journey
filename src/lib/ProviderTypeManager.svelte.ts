import { SvelteMap } from "svelte/reactivity";

import type { ProviderKey, ProviderType, ProviderDTO } from "./bindings.ts";

export class ProviderTypeProxy {
  keys = $state<(ProviderKey | undefined)[]>([]);

  constructor(keys: (ProviderKey | undefined)[] = []) {
    this.keys = keys;
  }

  addKey(key?: ProviderKey) {
    this.keys.push(key);
  }

  setKey(index: number, key: ProviderKey) {
    this.keys[index] = key;
  }
}

export class ProviderTypeManager {
  #types = new SvelteMap<ProviderType, ProviderTypeProxy>();
  #inProgress = $state(0);

  constructor(providers: Iterable<ProviderDTO> = []) {
    for (const { type, key } of providers) this.add(type, key);
  }

  get all() {
    return this.#types;
  }

  get knownTypes() {
    return [...this.#types.keys()];
  }

  get progress() {
    return this.#inProgress;
  }

  get(type: ProviderType) {
    return this.#types.get(type);
  }

  add(type: ProviderType, key?: ProviderKey) {
    this.#types.getOrInsert(type, new ProviderTypeProxy([key]));
  }

  incProgress() {
    this.#inProgress += 1;
  }

  decProgress() {
    this.#inProgress -= 1;
  }
}
