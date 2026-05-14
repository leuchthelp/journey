interface IMediaItem {
  backgroundImage: string;
  content: IMediaItem | undefined;
  outlineGradient: string;
  defaultStyling: string;
  animation: any;
}

interface IMediaItemFunctions {
  getBackgroundImage(): string;
  getContent(): any;
  getOutlineGradient(): string;
  getDefaultStyling(): string;
  getAnimation(): any;
}

export class SongItem implements IMediaItem, IMediaItemFunctions {
  backgroundImage = "";
  content: undefined;
  outlineGradient = "ring-[#C2381D]";
  defaultStyling = "m-0.5 h-24 w-24 rounded-full bg-amber-200 ring";
  animation = "";

  getBackgroundImage(): string {
    return this.backgroundImage;
  }

  getContent(): IMediaItem | undefined {
    return this.content;
  }

  getOutlineGradient(): string {
    return this.outlineGradient;
  }

  getDefaultStyling(): string {
    return this.defaultStyling;
  }

  getAnimation() {
    return this.animation;
  }
}
