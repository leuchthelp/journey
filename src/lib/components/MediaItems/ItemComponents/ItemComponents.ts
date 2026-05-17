import { SongItem, GenreItem, ArtistItem, PlaylistItem } from "../MediaItems";
import SongItemComponent from "./SongItemComponent.svelte";
import GenreItemComponent from "./GenreItemComponent.svelte";
import ArtistItemComponent from "./ArtistItemComponent.svelte";
import PlaylistItemComponent from "./PlaylistItemComponent.svelte";

export const componentOptions = new Map([
  [SongItem.name, SongItemComponent],
  [GenreItem.name, GenreItemComponent],
  [ArtistItem.name, ArtistItemComponent],
  [PlaylistItem.name, PlaylistItemComponent],
]);
