<script lang="ts">
  import type { MediaItem } from "$lib/db/relations";

  type Props = {
    item: MediaItem;
  };

  let { item }: Props = $props();

  let image = $derived(item.images.at(0)?.url!);
  let name = $derived(
    item.content.filter((item) => item.type === "Name").at(0),
  );
  let artist = $derived(
    item.content.filter((item) => item.type === "Artists").at(0),
  );
</script>

<div class="flex flex-col md:size-44 size-24">
  <button
    class={`m-0.5 rounded-xl bg-amber-200 ring-4 aspect-square overflow-x-clip  ${item.outlineGradient}`}
    aria-labelledby={name?.description}
  >
    <enhanced:img
      src={image}
      class="object-cover h-full w-full"
      alt="waiting"
    />
  </button>
  <div class="flex flex-col *:truncate">
    <span>{name?.description}</span>
    <span class="text-sm">{artist?.description}</span>
  </div>
</div>
