<script module lang="ts">
  import ProviderAccordionItem from "#lib/components/Settings/Provider/ProviderAccordionItem.svelte";
  import { type ProviderAuthSchema } from "../bindings";
  import { supportedAuthSchema } from "#lib/components/Settings/Auth/index.ts";
  import { type ProviderKey, type ProviderVariant } from "#lib/bindings.ts";
  export { toAuthComponent };
</script>

{#snippet toAuthComponent(
  schema: ProviderAuthSchema,
  type: ProviderVariant,
  key?: ProviderKey,
)}
  <ProviderAccordionItem>
    {#if supportedAuthSchema.has(schema)}
      {@const SvelteComponent = supportedAuthSchema.get(schema)}
      <SvelteComponent {type} {key}></SvelteComponent>
    {/if}
  </ProviderAccordionItem>
{/snippet}
