<script module lang="ts">
  import ProviderAccordionItem from "$lib/components/Settings/Provider/ProviderAccordionItem.svelte";
  import { providerAuthOptions } from "$lib/components/Settings/Provider/variants";
  export { toAuthComponent };
</script>

{#snippet toAuthComponent(type: string, id?: string)}
  <ProviderAccordionItem>
    {#if providerAuthOptions.has(type)}
      {@const SvelteComponent = providerAuthOptions.get(type)}
      <SvelteComponent {id}></SvelteComponent>
    {/if}
  </ProviderAccordionItem>
{/snippet}
