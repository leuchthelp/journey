<script>
  let { children } = $props();
  import "../app.css";

  import Playbar from "$lib/components/Playbar/Playbar.svelte";
</script>

<main class="mt-5 h-full">
  {@render children()}
</main>
<Playbar />
