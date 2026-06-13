<script lang="ts">
  import type { PageProps } from "./$types";
  import { itemCache, homeCache } from "$lib/components/MediaItems/ItemCache";
  import { toMediaItemComponent } from "$lib/snippets/ToMediaItemComponent.svelte";
  import "@videojs/html/audio/player";
  import "@videojs/html/audio/minimal-skin";

  let { data }: PageProps = $props();

  $inspect(data);
</script>

<div class="flex">
  {#each data.post as item}
    {#if itemCache.set(item.uuid!, item) && homeCache.set(item.uuid!, item)}
      {@render toMediaItemComponent(item)}
    {/if}
  {/each}
</div>
