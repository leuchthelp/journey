import { createContext } from "svelte";

interface IndexingState {
  type: string;
  uuid: string;
}

export interface Indexing {
  value: IndexingState;
}

export const [getIndexing, setIndexing] = createContext<Indexing>();
