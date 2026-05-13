interface MediaItem {
  backgroundImage(): string;
  content(): any;
  outlineGradient(): string;
  defaultStyling(): string[];
  animation(): any;
}

export class SongItem implements MediaItem {
  backgroundImage(): string {
    return "";
  }

  content(): any | MediaItem {
    return "";
  }

  outlineGradient(): string {
    return "ring-[#C2381D]";
  }

  defaultStyling(): string[] {
    return ["m-0.5", "h-24", "w-24", "rounded-full", "bg-amber-200"];
  }

  animation() {
    return;
  }
}
