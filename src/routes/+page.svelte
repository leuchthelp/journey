<script lang="ts">
  import { db } from "../db/InitDB";
  import { SongItem } from "$lib/components/MediaItems/MediaItems";
  import Playbar from "$lib/components/Playbar/Playbar.svelte";
  import SongItemComponent from "$lib/components/MediaItems/SongItemComponent.svelte";

  await db.deleteAll();

  const tmp = new SongItem();
  const test = ["testItem", JSON.stringify(tmp)];
  console.log(tmp);
  await db.addEntries(test);

  let res1 = $state(await db.getEntries());
  $inspect(res1);

  function parseObject(item: any) {
    const test = Object.assign(new SongItem(), JSON.parse(item["details"]));

    console.log(test);
    return test;
  }
</script>

<main class="mt-5 h-full">
  {#each res1 as item}
    <SongItemComponent songItem={parseObject(item)} />
  {/each}
</main>
<Playbar />
