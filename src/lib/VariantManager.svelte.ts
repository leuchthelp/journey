import { SvelteMap } from "svelte/reactivity";
import type { ProviderKey, ProviderVariant, ProviderDTO } from "./bindings.ts";

export class VariantProxy {
  name: ProviderVariant;
  keys = $state<(ProviderKey | undefined)[]>([]);

  constructor(name: ProviderVariant, keys: (ProviderKey | undefined)[] = []) {
    this.name = name;
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
    return this.#variants.values();
  }

  get knownVariants() {
    return [...this.#variants.values().map((value) => value.name)];
  }

  get(name: ProviderVariant) {
    return this.#variants.get(name);
  }

  add(name: ProviderVariant, key?: ProviderKey) {
    let variant = this.#variants.get(name);

    if (!variant) {
      variant = new VariantProxy(name, [key]);
      this.#variants.set(name, variant);
    } else if (key !== undefined) {
      variant.addKey(key);
    }
  }
}
