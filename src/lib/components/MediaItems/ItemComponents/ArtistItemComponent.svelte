<script lang="ts">
  import Button from "../MediaItemButton.svelte";
  import { ArtistItem } from "../MediaItems";

  type Props = {
    item: ArtistItem;
  };

  let { item }: Props = $props();
  let styling = $derived(item.defaultStyling + " " + item.outlineGradient);
  let content = $derived(item.content);
</script>

<Button {styling}><a href="/artist/{item.hash}">{content}</a></Button>
