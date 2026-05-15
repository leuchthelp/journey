<script lang="ts">
  import { MediaCache } from "../db/MediaCache";
  import Playbar from "$lib/components/Playbar/Playbar.svelte";
  import { SongItem } from "$lib/components/MediaItems/MediaItems";
  import SongItemComponent from "$lib/components/MediaItems/SongItemComponent.svelte";

  const cache = new MediaCache();

  const test = ["testItem", JSON.stringify(new SongItem())];

  //await cache.deleteAll();
  const res = await cache.addEntries(test);
  const res1 = await cache.getEntries();

  let test2: SongItem = Object.assign(
    new SongItem(),
    JSON.parse(res1[0]["details"]),
  );
</script>

<main class="mt-5 h-full">
  <SongItemComponent songItem={test2} />
</main>
<Playbar />
