import {
  AlbumItem,
  ArtistItem,
  GenreItem,
  PlaylistItem,
  SongItem,
} from "$lib/components/MediaItems/MediaItems.ts";
import { JellyfinProvider } from "./JellyfinProvider.ts";
import { BaseItemKind } from "@jellyfin/sdk/lib/generated-client/models";

export const providerOptions = new Map([
  ["JellyfinProvider", JellyfinProvider],
]);

export const mapJellyfinOptions = new Map([
  [`${BaseItemKind.Audio}`, SongItem],
  [`${BaseItemKind.Genre}`, GenreItem],
  [`${BaseItemKind.MusicArtist}`, ArtistItem],
  [`${BaseItemKind.MusicAlbum}`, AlbumItem],
  [`${BaseItemKind.Playlist}`, PlaylistItem],
]);
