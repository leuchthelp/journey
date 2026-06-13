import {
  AlbumItem,
  ArtistItem,
  GenreItem,
  PlaylistItem,
  SongItem,
} from "../MediaItems.ts";
import SongItemComponent from "./SongItemComponent.svelte";
import GenreItemComponent from "./GenreItemComponent.svelte";
import ArtistItemComponent from "./ArtistItemComponent.svelte";
import AlbumItemComponent from "./AlbumItemComponent.svelte";
import PlaylistItemComponent from "./PlaylistItemComponent.svelte";

export const componentOptions = new Map([
  [SongItem.name, SongItemComponent],
  [GenreItem.name, GenreItemComponent],
  [ArtistItem.name, ArtistItemComponent],
  [AlbumItem.name, AlbumItemComponent],
  [PlaylistItem.name, PlaylistItemComponent],
]);
