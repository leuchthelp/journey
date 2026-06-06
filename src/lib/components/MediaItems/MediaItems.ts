import type { ParentItem, MediaItem } from "../../db/relations";
import type {
  ContentItem,
  ProviderItem,
  ImageItem,
} from "../../db/schema/schema";

export class SongItem implements MediaItem {
  type = SongItem.name;
  outlineGradient = "ring-[#C2381D]";

  uuid!: string;
  defaultStyling!: string;
  loaded!: boolean;
  local!: string | null;

  content: ContentItem[] = [];
  providers: ProviderItem[] = [];
  images: ImageItem[] = [];
  parents: ParentItem[] = [];
}

export class ArtistItem implements MediaItem {
  type = ArtistItem.name;
  outlineGradient = "ring-[#D42CA4]";

  uuid!: string;
  defaultStyling!: string;
  loaded!: boolean;
  local!: string | null;

  content: ContentItem[] = [];
  providers: ProviderItem[] = [];
  images: ImageItem[] = [];
  parents: ParentItem[] = [];
}

export class GenreItem implements MediaItem {
  type = GenreItem.name;
  outlineGradient = "ring-[#2C8FD4]";

  uuid!: string;
  defaultStyling!: string;
  loaded!: boolean;
  local!: string | null;

  content: ContentItem[] = [];
  providers: ProviderItem[] = [];
  images: ImageItem[] = [];
  parents: ParentItem[] = [];
}

export class PlaylistItem implements MediaItem {
  type = PlaylistItem.name;
  outlineGradient = "ring-[#42D42C]";

  uuid!: string;
  defaultStyling!: string;
  loaded!: boolean;
  local!: string | null;

  content: ContentItem[] = [];
  providers: ProviderItem[] = [];
  images: ImageItem[] = [];
  parents: ParentItem[] = [];
}

export class AlbumItem implements MediaItem {
  type = AlbumItem.name;
  outlineGradient = "ring-[#D42CA4]";

  uuid!: string;
  defaultStyling!: string;
  loaded!: boolean;
  local!: string | null;

  content: ContentItem[] = [];
  providers: ProviderItem[] = [];
  images: ImageItem[] = [];
  parents: ParentItem[] = [];
}