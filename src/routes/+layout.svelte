<script lang="ts">
  let { children } = $props();
  import "../app.css";

  import Navbar from "$lib/components/Navbar/Navbar.svelte";
  import NavbarButton from "$lib/components/Navbar/NavbarButton.svelte";
  import Playbar from "$lib/components/Playbar/Playbar.svelte";
  import PlaybarButton from "$lib/components/Playbar/PlaybarButton.svelte";
  import ProviderAccordion from "$lib/components/Settings/Provider/ProviderAccordion.svelte";
  import ProviderAccordionItem from "$lib/components/Settings/Provider/ProviderAccordionItem.svelte";
  import Settings from "$lib/components/Settings/Settings.svelte";
  import JellyfinAuth from "$lib/components/Settings/Provider/Jellyfin/JellyfinAuth.svelte";

  let visible = $state(false);

  function toggleVisible() {
    visible = !visible;
  }
</script>

<main class="mt-5 h-full">
  {@render children()}
</main>
<Playbar>
  <PlaybarButton>1</PlaybarButton>
  <PlaybarButton>2</PlaybarButton>
  <PlaybarButton>3</PlaybarButton>
</Playbar>

<div class="fixed flex flex-row h-full place-self-start" class:w-full={visible}>
  <Navbar>
    <div class="h-full"></div>
    <NavbarButton func={toggleVisible}>settings</NavbarButton>
  </Navbar>
  {#if visible}
    <Settings>
      <ProviderAccordion title={"Providers"}>
        <ProviderAccordion title={"Jellyfin"}>
          <ProviderAccordionItem>
            <JellyfinAuth></JellyfinAuth>
          </ProviderAccordionItem>
        </ProviderAccordion>
      </ProviderAccordion>
    </Settings>
  {/if}
</div>
