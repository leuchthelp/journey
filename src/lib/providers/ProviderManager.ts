import type { Provider } from "./variants/Provider";

type IProviderManager = {
  providers: Provider[];
};

export class ProviderManager implements IProviderManager {
  providers: Provider[] = [];
}
