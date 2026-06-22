<script lang="ts">
  import { itemCache } from "$lib/components/MediaItems/ItemCache.ts";
  import ItemComponent from "$lib/components/MediaItems/ItemComponent.svelte";
  import type { MediaItem } from "$lib/db/relations.ts";
  import { getIndexing } from "$lib/signals/index.svelte";
  import { mainPageDataQuery } from "$lib/db/queries";

  let data: MediaItem[] = $state([]);
  let type = $state("SongItem");
  let limit = $state(6);

  let signal = getIndexing();
  $inspect(signal);

  $effect(() => {
    async () => {
      if (signal.type === type) {
        data = await mainPageDataQuery.execute({ limit, type });
      }
    };
  });
</script>

<div class="flex">
  {#each data as item}
    {#if itemCache.set(item.uuid!, item)}
      <ItemComponent {item}></ItemComponent>
    {/if}
  {/each}
</div>
