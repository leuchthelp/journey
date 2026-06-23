<script lang="ts">
  import { invalidate } from "$app/navigation";
  import { itemCache } from "$lib/components/MediaItems/ItemCache.ts";
  import ItemComponent from "$lib/components/MediaItems/ItemComponent.svelte";
  import { getIndexing } from "$lib/signals/index.svelte";
  import type { PageProps } from "./$types";

  let { data }: PageProps = $props();

  let type = $state("SongItem");
  let limit = $state(6);
  let signal = getIndexing();

  $effect(() => {
    if (signal.value.type === type && data.post.length < limit) {
      console.warn("reload page");
      invalidate("app:mainPage");
    }
  });
</script>

<div class="flex gap-3">
  {#each data.post as item}
    {#if itemCache.set(item.uuid, item)}
      <ItemComponent {item}></ItemComponent>
    {/if}
  {/each}
</div>
