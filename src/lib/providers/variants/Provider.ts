export type Provider = {
  readonly client: unknown;

  createApi?: (url: string, token?: string) => unknown;
  getApi?: () => unknown;
  getServerID?: () => string | undefined;
};
