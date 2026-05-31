import type { ProviderItem } from "$lib/db/schema/schema";

export interface Provider extends ProviderItem {
  readonly client: unknown;

  createApi: (url: string, token?: string) => unknown;

  authApiWithPw?: (uname: string, psw: string) => unknown;
  authStatus: () => boolean;

  addToDB: () => void;
  retrieveCredentials: () => void;

  getApi: () => unknown;
  getID: () => string | undefined;

  setID: (id: string) => void;
  setURL: (url: string) => void;
}
