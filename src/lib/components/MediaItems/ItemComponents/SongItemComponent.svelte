<script lang="ts">
  import Button from "../MediaItemButton.svelte";
  import { SongItem } from "../MediaItems";

  type Props = {
    item: SongItem;
  };

  let { item: songItem }: Props = $props();
  let styling = $derived(
    songItem.defaultStyling + " " + songItem.outlineGradient,
  );
  let content = $derived(songItem.content);
</script>

<Button {styling}>{content}</Button>
