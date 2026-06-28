import { createPlayer, audioFeatures, MediaElement } from "@videojs/html";

const Player = createPlayer({ features: audioFeatures });
const store = Player.create();

class AudioPlayer extends Player.ProviderMixin(MediaElement) {
  static readonly tagName = "journey-audio-player";
}

export { store, Player, AudioPlayer };
