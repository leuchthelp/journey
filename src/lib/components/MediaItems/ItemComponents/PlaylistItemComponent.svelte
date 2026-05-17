<script lang="ts">
  import Button from "../MediaItemButton.svelte";
  import { PlaylistItem } from "../MediaItems";

  type Props = {
    item: PlaylistItem;
  };

  let { item: songItem }: Props = $props();
  let styling = $derived(
    songItem.defaultStyling + " " + songItem.outlineGradient,
  );
  let content = $derived(songItem.content);
</script>

<Button {styling}>{content}</Button>
