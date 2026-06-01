import { providerOptions } from "./variants";
import type { ProviderItem } from "$lib/db/schema/schema";
import type { Provider } from "./variants/Provider";

type IProviderManager = {
  providers: Provider[];

  getProviderByID: (serverId: string) => Provider;
  addProvider: (provider: Provider) => void;
  removeProvider: (provider: Provider) => void;
  initProvider: (provider: Provider) => void;
  existsProviderWith: (userId: string) => boolean;
};

export class ProviderManager implements IProviderManager {
  providers: Provider[] = [];

  public getProviderByID(serverId: string): Provider {
    return this.providers
      .filter((provider) => provider.getServerId() === serverId)
      .at(0)!;
  }

  public existsProviderWith(userId: string): boolean {
    let tmp = this.providers.filter((provider) => provider.userId === userId);
    return tmp.length > 0 ? true : false;
  }

  public addProvider(provider: Provider) {
    this.providers.push(provider);
  }

  public removeProvider(provider: Provider) {
    let index = this.providers.indexOf(provider);
    delete this.providers[index];
  }

  public initProvider(providerItem: ProviderItem) {
    if (providerOptions.has(providerItem.type)) {
      let accessToken =
        localStorage.getItem(`${providerItem.serverId}Token`) || undefined;

      let provider = providerOptions.get(providerItem.type)!;
      let init = new provider(
        providerItem.serverId,
        providerItem.url,
        providerItem.userId,
        accessToken,
      );
      this.addProvider(init);
    }
  }
}

export const providerManager = new ProviderManager();
