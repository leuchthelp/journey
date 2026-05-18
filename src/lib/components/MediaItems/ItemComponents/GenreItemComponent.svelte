<script lang="ts">
  import Button from "../MediaItemButton.svelte";
  import { GenreItem } from "../MediaItems";

  type Props = {
    item: GenreItem;
  };

  let { item }: Props = $props();
  let styling = $derived(item.defaultStyling + " " + item.outlineGradient);
  let content = $derived(item.content);
</script>

<Button {styling}><a href="/genre/{item.hash}">{content}</a></Button>
