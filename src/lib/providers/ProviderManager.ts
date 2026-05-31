import { providerOptions } from "./variants";
import type { ProviderItem } from "$lib/db/schema/schema";
import type { Provider } from "./variants/Provider";

type IProviderManager = {
  providers: Provider[];

  getProviderByID: (id: string) => Provider;
  addProvider: (provider: Provider) => void;
  initProvider: (provider: Provider) => void;
};

export class ProviderManager implements IProviderManager {
  providers: Provider[] = [];

  public getProviderByID(id: string): Provider {
    return this.providers.filter((provider) => provider.getID!() === id).at(0)!;
  }

  public addProvider(provider: Provider) {
    this.providers.push(provider);
  }

  public initProvider(providerItem: ProviderItem) {
    if (providerOptions.has(providerItem.type)) {
      let accessToken =
        localStorage.getItem(`${providerItem.id}Token`) || undefined;

      let provider = providerOptions.get(providerItem.type)!;
      let init = new provider(providerItem.id, providerItem.url, accessToken);
      console.log(init);
      this.addProvider(init);
    }
  }
}

export const providerManager = new ProviderManager();
