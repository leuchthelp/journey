import { createContext } from "svelte";

interface MediaSourceState {
  url: string;
}

export const [getMediaSource, setMediaSource] =
  createContext<MediaSourceState>();
