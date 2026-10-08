<script lang="ts">
  import { type MediaItemType } from "#lib/bindings.ts";

  type Props = {
    type: MediaItemType;
    uuid: string;
    description?: string;
    children?: import("svelte").Snippet;
  };

  let { type, uuid, description, children }: Props = $props();
</script>

<a href="/pages/{type}/{uuid}" class="flex" aria-labelledby={description}>
  {@render children?.()}
</a>
