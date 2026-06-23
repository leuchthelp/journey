<script lang="ts">
  type Props = {
    children?: import("svelte").Snippet;
  };

  let { children }: Props = $props();
</script>

<div
  class="flex flex-col md:flex-col-reverse rounded-2xl m-2 p-3 pr-15 backdrop-blur-sm backdrop-saturate-125 shadow-xl outline-2 outline-gray-400"
>
  {@render children?.()}
</div>
