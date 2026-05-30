<script module lang="ts">
  import { componentOptions } from "$lib/components/MediaItems/ItemComponents";
  import type { MediaItems } from "$lib/db/schema/schema";
  export { toSvelteComponent };
</script>

{#snippet toSvelteComponent(item: MediaItems)}
  {#if componentOptions.has(item.type)}
    {@const SvelteComponent = componentOptions.get(item.type)}
    <SvelteComponent {item}></SvelteComponent>
  {/if}
{/snippet}
