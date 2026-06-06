import type { ParentItem, MediaItem } from "../../db/relations";
import type {
  ContentItem,
  ProviderItem,
  ImageItem,
} from "../../db/schema/schema";

class BaseItem implements MediaItem {
  type!: string;
  outlineGradient!: string;

  uuid!: string;
  defaultStyling!: string;
  loaded!: boolean;
  local!: string;

  content: ContentItem[] = [];
  providers: ProviderItem[] = [];
  images: ImageItem[] = [];
  parents: ParentItem[] = [];
}

export class SongItem extends BaseItem {
  override type = SongItem.name;
  override outlineGradient = "ring-[#C2381D]";
}

export class ArtistItem extends BaseItem {
  override type = ArtistItem.name;
  override outlineGradient = "ring-[#D42CA4]";
}

export class GenreItem extends BaseItem {
  override type = GenreItem.name;
  override outlineGradient = "ring-[#2C8FD4]";
}

export class PlaylistItem extends BaseItem {
  override type = PlaylistItem.name;
  override outlineGradient = "ring-[#42D42C]";
}

export class AlbumItem extends BaseItem {
  override type = AlbumItem.name;
  override outlineGradient = "ring-[#D42CA4]";
}
