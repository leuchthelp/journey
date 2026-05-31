import type { MediaItem } from "../../db/schema/schema.ts";

export class SongItem implements Omit<MediaItem, "id"> {
  type = SongItem.name;
  outlineGradient = "ring-[#C2381D]";

  hash!: string;
  backgroundImage!: string;
  content!: string;
  defaultStyling!: string;
  animation!: string;
  loaded!: boolean;
  local!: string;
  providers!: string;
}

export class ArtistItem implements Omit<MediaItem, "id"> {
  type = ArtistItem.name;
  outlineGradient = "ring-[#D42CA4]";

  hash!: string;
  backgroundImage!: string;
  content!: string;
  defaultStyling!: string;
  animation!: string;
  loaded!: boolean;
  local!: string;
  providers!: string;
}

export class GenreItem implements Omit<MediaItem, "id"> {
  type = GenreItem.name;
  outlineGradient = "ring-[#2C8FD4]";

  hash!: string;
  backgroundImage!: string;
  content!: string;
  defaultStyling!: string;
  animation!: string;
  loaded!: boolean;
  local!: string;
  providers!: string;
}

export class PlaylistItem implements Omit<MediaItem, "id"> {
  type = PlaylistItem.name;
  outlineGradient = "ring-[#42D42C]";

  hash!: string;
  backgroundImage!: string;
  content!: string;
  defaultStyling!: string;
  animation!: string;
  loaded!: boolean;
  local!: string;
  providers!: string;
}
