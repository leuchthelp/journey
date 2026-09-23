import { SvelteMap } from "svelte/reactivity";
import type { ProviderKey, ProviderVariant, ProviderDTO } from "./bindings.ts";

export class VariantProxy {
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

export class VariantManager {
  #variants = new SvelteMap<ProviderVariant, VariantProxy>();

  constructor(providers: Iterable<ProviderDTO> = []) {
    for (const { type, key } of providers) this.add(type, key);
  }

  get all() {
    return this.#variants;
  }

  get knownVariants() {
    return [...this.#variants.keys()];
  }

  get(variant: ProviderVariant) {
    return this.#variants.get(variant);
  }

  add(variant: ProviderVariant, key?: ProviderKey) {
    this.#variants.getOrInsert(variant, new VariantProxy([key]));
  }
}
