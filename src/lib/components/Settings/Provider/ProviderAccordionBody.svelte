<script lang="ts">
  import ProviderAccordion from "#lib/components/Settings/Provider/ProviderAccordion.svelte";
  import { Effect } from "effect";
  import { getSupportedAuthSchema } from "#lib/effects/provider.ts";
  import ProviderAccordionItem from "#lib/components/Settings/Provider/ProviderAccordionItem.svelte";
  import { supportedAuthSchema } from "../Auth";
  import type { ProviderVariant, ProviderKey } from "#lib/bindings.ts";

  type Props = {
    variant: ProviderVariant;
    knownKeys: (ProviderKey | undefined)[];
    addVariant: (type: ProviderVariant, provider?: ProviderKey) => void;
  };
  let { variant, knownKeys = $bindable(), addVariant }: Props = $props();
</script>

<ProviderAccordion title={variant}>
  <button onclick={() => addVariant(variant)}>Add new {variant}</button>
  {#each knownKeys as _, i}
    {#await Effect.runPromise(getSupportedAuthSchema(variant)) then supportedSchema}
      {#each supportedSchema as schema}
        <ProviderAccordionItem>
          {#if supportedAuthSchema.has(schema)}
            {@const SvelteComponent = supportedAuthSchema.get(schema)}
            <SvelteComponent {variant} bind:key={knownKeys[i]}
            ></SvelteComponent>
          {/if}
        </ProviderAccordionItem>
      {/each}
    {/await}
  {/each}
</ProviderAccordion>
