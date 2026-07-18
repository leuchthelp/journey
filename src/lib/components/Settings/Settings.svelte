<script lang="ts">
  type Props = {
    children?: import("svelte").Snippet;
  };

  let { children }: Props = $props();
</script>

<div
  class="flex grow flex-col rounded-2xl p-3 shadow-xl outline-2 outline-gray-400 backdrop-blur-sm backdrop-saturate-125"
>
  {@render children?.()}
</div>
