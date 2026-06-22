import { createContext } from "svelte";

export interface Indexing {
  type: string;
  uuid: string;
}

export const [getIndexing, setIndexing] = createContext<Indexing>();
