<script module lang="ts">
  import { componentOptions } from "$lib/components/MediaItems/ItemComponents";
  import type { MediaItem } from "$lib/db/relations";
  export { toMediaItemComponent };
</script>

{#snippet toMediaItemComponent(item: MediaItem)}
  {#if componentOptions.has(item.type)}
    {@const SvelteComponent = componentOptions.get(item.type)}
    <SvelteComponent {item}></SvelteComponent>
  {/if}
{/snippet}
