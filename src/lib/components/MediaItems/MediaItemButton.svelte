<script lang="ts">
  import type { MediaItem } from "$lib/db/relations";

  type Props = {
    item: MediaItem;
  };

  let { item }: Props = $props();

  let image = $derived(item.images.at(0)?.url!);
  let styling = $derived("m-0.5 h-24 w-24 rounded-xl bg-amber-200 ring" + " " + item.outlineGradient + " overflow-hidden");
  let name = $derived(
    item.content.filter((item) => item.type === "Name").at(0),
  );
  let artist = $derived(
    item.content.filter((item) => item.type === "Artists").at(0),
  );
</script>

<button class={styling} onclick={() => console.log(styling)}>
  <img src={image} class="object-cover h-full w-full" alt="waiting" />
</button>
<div>{name?.description}</div>
<div>{artist?.description}</div>
