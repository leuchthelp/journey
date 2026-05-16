import { type MediaItems } from "../../../db/schema";

export class SongItem implements Omit<MediaItems, "id"> {
  hash = null;
  backgroundImage = null;
  content = null;
  outlineGradient = "ring-[#C2381D]";
  defaultStyling = "m-0.5 h-24 w-24 rounded-full bg-amber-200 ring";
  animation = null;
  loaded = false;
  local = null;
  providers = "{}";
}

export class ArtistItem implements Omit<MediaItems, "id"> {
  hash = null;
  backgroundImage = null;
  content = null;
  outlineGradient = "ring-[#C2381D]";
  defaultStyling = "m-0.5 h-24 w-24 rounded-full bg-amber-200 ring";
  animation = null;
  loaded = false;
  local = null;
  providers = "{}";
}
