<script lang="ts">
  import ItemComponent from "#lib/components/MediaItems/ItemComponent.svelte";
  import type { PageProps } from "./$types";

  let { data }: PageProps = $props();
</script>

<div class="flex gap-3">
  {#await data.mediaItemsReq then mediaItems}
    {#each mediaItems as item}
      <ItemComponent {item}></ItemComponent>
    {/each}
  {/await}
</div>
