import type { ProviderItem } from "$lib/db/schema/schema.ts";
import type { Indexing } from "../../signals/index.svelte.ts";

export interface Provider extends Omit<ProviderItem, "id"> {
  readonly client: unknown;

  createApi: (url: string, token?: string) => void;

  authApiWithPw?: (uname: string, psw: string) => Promise<void>;
  authStatus: () => boolean;
  setAuthStatus: (value: boolean) => void;

  addToDb: () => Promise<void>;
  retrieveCredentials?: () => void;
  removeConnection: () => void;

  getApi: () => unknown;
  getServerId: () => string;
  getUserId: () => string;

  indexFiles: (signal: Indexing) => Promise<void>;
}
