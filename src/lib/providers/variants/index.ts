import {
  ArtistItem,
  GenreItem,
  PlaylistItem,
  SongItem,
} from "../../components/MediaItems/MediaItems";
import { JellyfinProvider } from "./JellyfinProvider";
import { BaseItemKind } from "@jellyfin/sdk/lib/generated-client/models";

export const providerOptions = new Map([
  ["JellyfinProvider", JellyfinProvider],
]);

export const mapJellyfinOptions = new Map([
  [`${BaseItemKind.Audio}`, SongItem],
  [`${BaseItemKind.Genre}`, GenreItem],
  [`${BaseItemKind.MusicArtist}`, ArtistItem],
  [`${BaseItemKind.Playlist}`, PlaylistItem],
]);
