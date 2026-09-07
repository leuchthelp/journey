import { createPlayer, audioFeatures, MediaElement } from "@videojs/html";

const { ProviderMixin, ContainerMixin, create } = createPlayer({
  features: audioFeatures,
});
const store = create();

class AudioPlayer extends ProviderMixin(ContainerMixin(MediaElement)) {
  static readonly tagName = "journey-audio-player";
}
customElements.define("journey-audio-player", AudioPlayer);

export { store, AudioPlayer };
