import { providerOptions } from "./variants/index.ts";
import type { ProviderItem } from "$lib/db/schema/schema.ts";
import type { Provider } from "./variants/Provider.ts";

type IProviderManager = {
  providers: Provider[];
  indexPromises: Promise<void>[];

  getProviderByServerId: (serverId: string) => Provider;
  addProvider: (provider: Provider) => void;
  removeProvider: (provider: Provider) => void;
  initProvider: (providerItems: Provider[]) => void;
  existsProviderWith: (userId: string) => boolean;
};

class ProviderManager implements IProviderManager {
  providers: Provider[] = [];
  indexPromises: Promise<void>[] = [];

  public getProviderByServerId(serverId: string): Provider {
    return this.providers
      .filter((provider) => provider.getServerId() === serverId)
      .at(0)!;
  }

  public getProviderByUserId(userId: string): Provider {
    return this.providers
      .filter((provider) => provider.getUserId() === userId)
      .at(0)!;
  }

  public existsProviderWith(userId: string): boolean {
    const tmp = this.providers.filter((provider) => provider.userId === userId);
    return tmp.length > 0 ? true : false;
  }

  public addProvider(provider: Provider) {
    this.providers.push(provider);
  }

  public removeProvider(provider: Provider) {
    const index = this.providers.indexOf(provider);
    delete this.providers[index];
  }

  public initProvider(providerItems: ProviderItem[]) {
    providerItems.forEach((providerItem) => {
      if (providerOptions.has(providerItem.type)) {
        const accessToken =
          localStorage.getItem(`${providerItem.serverId}Token`) || undefined;

        const provider = providerOptions.get(providerItem.type)!;
        const init = new provider(
          providerItem.serverId,
          providerItem.url,
          providerItem.userId,
          accessToken,
        );
        this.addProvider(init);

        if (init.authStatus()) {
          this.indexPromises.push(init.indexFiles());
        }
      }
    });

    Promise.allSettled(this.indexPromises).finally(() =>
      console.log("indexing finished"),
    );
  }
}

export const providerManager = new ProviderManager();
