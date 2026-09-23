<script lang="ts">
  import { Effect } from "effect";
  import * as ProviderAccordion from "#lib/components/Settings/Provider/index.ts";
  import { getSupportedAuthSchema } from "#lib/effects/provider.ts";
  import { authSchemaComponents } from "../Auth/index.ts";
  import type { VariantProxy } from "#lib/VariantManager.svelte.ts";
  import type { ProviderKey, ProviderVariant } from "#lib/bindings.ts";

  type Props = {
    variant: ProviderVariant;
    proxy: VariantProxy;
  };
  let { variant, proxy }: Props = $props();
</script>

<ProviderAccordion.Root title={variant}>
  <button onclick={() => proxy.addKey()}>Add new {variant}</button>
  {#each proxy.keys as key, i (i)}
    {#await Effect.runPromise(getSupportedAuthSchema(variant)) then supportedSchema}
      {#each supportedSchema as schema}
        {#if authSchemaComponents.has(schema)}
          {@const AuthComponent = authSchemaComponents.get(schema)}
          <AuthComponent
            {variant}
            {key}
            onKeyChange={(v: ProviderKey) => proxy.setKey(i, v)}
          ></AuthComponent>
        {/if}
      {/each}
    {/await}
  {/each}
</ProviderAccordion.Root>
