<script lang="ts">
  import type { MediaItemDTO } from "#lib/bindings.ts";
  import { Effect } from "effect";
  import { stream } from "#lib/effects/provider.ts";
  import { Play } from "@lucide/svelte";
  import PageLink from "../PageLink.svelte";

  type Props = {
    item: MediaItemDTO;
  };

  let { item }: Props = $props();

  let name = $derived(item.content?.Name);
  let imageUrl = $derived(item.images?.Primary?.at(0)?.url);
</script>

<div class="flex size-[20vh] flex-col">
  <div class="group flex flex-row-reverse">
    <PageLink type={item.type} uuid={item.uuid} description={name?.description}>
      <enhanced:img
        src={imageUrl}
        class="m-0.5 aspect-square h-full w-full overflow-x-clip rounded-xl object-cover ring-4 {item.outlineGradient}"
        alt="waiting"
      />
    </PageLink>
    <button
      class="invisible absolute self-end-safe rounded-full bg-amber-50 group-hover:visible hover:bg-amber-500"
      onclick={async () =>
        await Effect.runPromise(
          stream(
            item.providers?.JellyfinProvider?.at(0)?.key!,
            item.sources?.at(0)?.sourceId!,
          ),
        )}><Play /></button
    >
  </div>
  <PageLink type={item.type} uuid={item.uuid} description={name?.description}>
    <span class="*:truncate">{name?.description}</span>
  </PageLink>
  <div class="flex flex-row *:truncate">
    {#each item.parents?.Artist as artist}
      <a
        href="/pages/{artist?.type}/{artist.uuid}"
        class="flex flex-col"
        aria-labelledby={artist.content?.Name?.description}
      >
        <span class="text-sm">{artist.content?.Name?.description}</span>
      </a>
    {/each}
  </div>
</div>
