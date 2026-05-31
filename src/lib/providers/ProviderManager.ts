import type { Provider } from "./variants/Provider";

type IProviderManager = {
  providers: Provider[];

  getProviderByID: (id: string) => Provider;
  addProvider: (provider: Provider) => void;
};

export class ProviderManager implements IProviderManager {
  providers: Provider[] = [];

  public getProviderByID(id: string): Provider {
    return this.providers
      .filter((provider) => provider.getID!() === id)
      .at(0)!;
  }

  public addProvider(provider: Provider) {
    this.providers.push(provider);
  }
}

export const providerManager = new ProviderManager();
