import type { ProviderItem } from "$lib/db/schema/schema";

export interface Provider extends ProviderItem {
  readonly client: unknown;

  createApi: (url: string, token?: string) => void;

  authApiWithPw?: (uname: string, psw: string) => void;
  authStatus: () => boolean;

  addToDB: () => Promise<void>;
  retrieveCredentials?: () => void;
  removeConnection: () => void;

  getApi: () => unknown;
  getID: () => string | undefined;
}
