import type { Provider } from "./variants/Provider";

type IProviderManager = {
  providers: Provider[];

  getProviderByServerID: (serverID: string) => Provider | undefined;
  addProvider: (provider: Provider) => void;
};

export class ProviderManager implements IProviderManager {
  providers: Provider[] = [];

  public getProviderByServerID(serverID: string): Provider | undefined {
    return this.providers
      .filter((provider) => provider.getServerID!() === serverID)
      .at(0);
  }

  public addProvider(provider: Provider) {
    this.providers.push(provider);
  }
}

export const providerManager = new ProviderManager();
