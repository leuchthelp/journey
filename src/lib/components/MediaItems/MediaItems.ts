import { type MediaItems } from "../../../db/schema";

export class SongItem implements Omit<MediaItems, "id"> {
  type = "SongItem";
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

export class ArtistItem implements Omit<MediaItems, "id"> {
  type = "ArtistItem";
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

export class GenreItem implements Omit<MediaItems, "id"> {
  type = "GenreItem";
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

export class PlaylistItem implements Omit<MediaItems, "id"> {
  type = "PlaylistItem";
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
