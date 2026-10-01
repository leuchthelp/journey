<script lang="ts">
  import { itemCache } from "#lib/components/MediaItems/ItemCache.ts";
  import ItemComponent from "#lib/components/MediaItems/ItemComponent.svelte";
  import type { PageProps } from "./$types";

  let { data }: PageProps = $props();
</script>

<div class="flex gap-3">
  {#await data.mediaItemsReq then mediaItems}
    {#each mediaItems as item}
      {#if itemCache.set(item.uuid, item)}
        <ItemComponent {item}></ItemComponent>
      {/if}
    {/each}
  {/await}
</div>
