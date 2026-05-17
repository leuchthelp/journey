<script lang="ts">
  import type { PageProps } from "./$types";

  let { data }: PageProps = $props();
</script>

<h1>
  {#if data.post[0] !== undefined}
    {data.post[0].hash}
  {/if}
</h1>
