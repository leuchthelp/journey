<script lang="ts">
  import Button from "../MediaItemButton.svelte";
  import { GenreItem } from "../MediaItems";

  type Props = {
    item: GenreItem;
  };

  let { item: songItem }: Props = $props();
  let styling = $derived(
    songItem.defaultStyling + " " + songItem.outlineGradient,
  );
  let content = $derived(songItem.content);
</script>

<Button {styling}><a href="/{songItem.hash}">{content}</a></Button>
