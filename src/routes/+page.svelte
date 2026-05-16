<script lang="ts">
  import { db } from "../db/database";
  import * as schema from "../db/schema";
  import { SongItem } from "$lib/components/MediaItems/MediaItems";
  import Playbar from "$lib/components/Playbar/Playbar.svelte";
  import SongItemComponent from "$lib/components/MediaItems/SongItemComponent.svelte";

  //await db.insert(schema.mediaItems).values(test)
  //await db.delete(schema.mediaItems)

  let res1 = await db.query.mediaItems.findMany().execute();

  function parseObject(item: schema.MediaItems) {
    return Object.assign(new SongItem(), item);
  }
</script>

<main class="mt-5 h-full">
  {#each res1 as item}
    <SongItemComponent songItem={parseObject(item)} />
  {/each}
</main>
<Playbar />
