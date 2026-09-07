import type {
  ContentDTO,
  ImageDTO,
  MediaItemDTO,
  SourceDTO,
  ProviderDTO,
  MediaItemType,
} from "../../bindings.ts";

class BaseItem implements MediaItemDTO {
  uuid!: string;
  isTmp!: boolean;
  type!: MediaItemType;
  outlineGradient!: string;

  sources: SourceDTO[] = [];
  content: ContentDTO[] = [];
  providers: ProviderDTO[] = [];
  images: ImageDTO[] = [];
  parents: MediaItemDTO[] = [];
  children: MediaItemDTO[] = [];
}

export class SongItem extends BaseItem {
  override type = "Audio" as MediaItemType;
  override outlineGradient = "ring-[#C2381D]";
}

export class ArtistItem extends BaseItem {
  override type = "Artist" as MediaItemType;
  override outlineGradient = "ring-[#D42CA4]";
}

export class GenreItem extends BaseItem {
  override type = "Genre" as MediaItemType;
  override outlineGradient = "ring-[#2C8FD4]";
}

export class PlaylistItem extends BaseItem {
  override type = "Playlist" as MediaItemType;
  override outlineGradient = "ring-[#42D42C]";
}

export class AlbumItem extends BaseItem {
  override type = "Album" as MediaItemType;
  override outlineGradient = "ring-[#D42CA4]";
}
