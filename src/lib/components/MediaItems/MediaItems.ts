import { MediaPaths } from "./MediaPaths";

interface IMediaItem {
  backgroundImage: string;
  content: string;
  outlineGradient: string;
  defaultStyling: string;
  animation: any;
  loaded: boolean;
  paths: MediaPaths;
}

export class SongItem implements IMediaItem {
  paths = new MediaPaths();
  backgroundImage = "";
  content = "";
  outlineGradient = "ring-[#C2381D]";
  defaultStyling = "m-0.5 h-24 w-24 rounded-full bg-amber-200 ring";
  animation = "";
  loaded = false;
}
