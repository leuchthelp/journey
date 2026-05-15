interface IMediaPaths {
  local: string;
  providers: Map<string, string>;
}

export class MediaPaths implements IMediaPaths {
  local = "";
  providers = new Map();
}
