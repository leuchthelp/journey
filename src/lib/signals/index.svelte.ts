import { createContext } from "svelte";

export interface Indexing {
  type: string;
}

export const [getIndexing, setIndexing] = createContext<Indexing>();
