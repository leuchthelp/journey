import type {
  ContentDTO,
  ImageDTO,
  MediaItemDTO,
  OriginalDTO,
  ProviderDTO,
} from "../../bindings.ts";

class BaseItem implements MediaItemDTO {
  type!: string;
  outlineGradient!: string;

  uuid!: string;
  loaded!: boolean;
  local!: string;

  original: OriginalDTO[] = [];
  content: ContentDTO[] = [];
  providers: ProviderDTO[] = [];
  images: ImageDTO[] = [];
  parents: MediaItemDTO[] = [];
  children: MediaItemDTO[] = [];
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
