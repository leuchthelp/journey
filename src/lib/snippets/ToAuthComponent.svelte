<script module lang="ts">
  import ProviderAccordionItem from "$lib/components/Settings/Provider/ProviderAccordionItem.svelte";
  import { providerAuthOptions } from "$lib/components/Settings/Auth/variants";
  export { toAuthComponent };
</script>

{#snippet toAuthComponent(type: string, serverId?: string)}
  <ProviderAccordionItem>
    {#if providerAuthOptions.has(type)}
      {@const SvelteComponent = providerAuthOptions.get(type)}
      <SvelteComponent {serverId}></SvelteComponent>
    {/if}
  </ProviderAccordionItem>
{/snippet}
