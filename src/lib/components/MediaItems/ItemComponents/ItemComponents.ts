import SongItemComponent from "./SongItemComponent.svelte";
import GenreItemComponent from "./GenreItemComponent.svelte";
import ArtistItemComponent from "./ArtistItemComponent.svelte";
import PlaylistItemComponent from "./PlaylistItemComponent.svelte";

export const componentOptions = new Map([
  ["SongItem", SongItemComponent],
  ["GenreItem", GenreItemComponent],
  ["ArtistItem", ArtistItemComponent],
  ["PlaylistItem", PlaylistItemComponent],
]);
