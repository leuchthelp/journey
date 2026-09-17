<script lang="ts">
  import ProviderAccordion from "#lib/components/Settings/Provider/ProviderAccordion.svelte";
  import { Effect } from "effect";
  import { getSupportedAuthSchema } from "#lib/effects/provider.ts";
  import { authSchemaComponents } from "../Auth";
  import type { ProviderVariant, ProviderKey } from "#lib/bindings.ts";

  type Props = {
    variant: ProviderVariant;
    knownKeys: (ProviderKey | undefined)[] | undefined;
  };
  let { variant, knownKeys = $bindable() }: Props = $props();
</script>

<ProviderAccordion title={variant}>
  {#if knownKeys !== undefined}
    <button onclick={() => knownKeys.push(undefined)}>Add new {variant}</button>
    {#each knownKeys as _, i}
      {#await Effect.runPromise(getSupportedAuthSchema(variant)) then supportedSchema}
        {#each supportedSchema as schema}
          {#if authSchemaComponents.has(schema)}
            {@const SvelteComponent = authSchemaComponents.get(schema)}
            <SvelteComponent {variant} bind:key={knownKeys[i]}
            ></SvelteComponent>
          {/if}
        {/each}
      {/await}
    {/each}
  {/if}
</ProviderAccordion>
