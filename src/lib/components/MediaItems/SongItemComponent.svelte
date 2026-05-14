<script lang="ts">
  import Button from "./MediaItemButton.svelte";
  import { SongItem } from "./MediaItems";

  type Props = {
    songItem: SongItem;
  };

  let { songItem }: Props = $props();
  let styling = $derived(songItem.defaultStyling + songItem.outlineGradient);

  $inspect(styling)
  let content = $derived(songItem.content);
</script>

<Button styling={styling}>{content}</Button>
