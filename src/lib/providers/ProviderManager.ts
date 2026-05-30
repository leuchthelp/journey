import type { Provider } from "./variants/Provider";

type IProviderManager = {
  providers: Provider[];

  getProviderByServerID: (serverID: string) => Provider | undefined;
};

export class ProviderManager implements IProviderManager {
  providers: Provider[] = [];

  public getProviderByServerID(serverID: string): Provider | undefined {
    return this.providers
      .filter((provider) => provider.getServerID!() === serverID)
      .at(0);
  }
}

export const providerManager = new ProviderManager();
