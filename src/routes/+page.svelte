<script lang="ts">
  import { db } from "../db/database";
  import * as schema from "../db/schema";
  import Playbar from "$lib/components/Playbar/Playbar.svelte";
  import { componentOptions } from "$lib/components/MediaItems/ItemComponents/ItemComponents";
  //import {
  //  ArtistItem,
  //  GenreItem,
  //  PlaylistItem,
  //  SongItem,
  //} from "$lib/components/MediaItems/MediaItems";
  //
  //const test = new SongItem();
  //const test2 = new ArtistItem();
  //const test3 = new GenreItem();
  //const test4 = new PlaylistItem();
  //
  //test.hash = "SongItem";
  //test2.hash = "ArtistItem";
  //test3.hash = "GenreItem";
  //test4.hash = "PlaylistItem";
  //
  //test.content = "SongItem";
  //test2.content = "ArtistItem";
  //test3.content = "GenreItem";
  //test4.content = "PlaylistItem";
  //await db.insert(schema.mediaItems).values([test, test2, test3, test4])
  //await db.delete(schema.mediaItems)

  let res1 = await db.query.mediaItems.findMany().execute();

  console.log(res1);
</script>

<main class="mt-5 h-full">
  {#each res1 as item}
    {#if componentOptions.has(item.type)}
      {@const SvelteComponent = componentOptions.get(item.type)}
      <SvelteComponent {item}></SvelteComponent>
    {/if}
  {/each}
</main>
<Playbar />
