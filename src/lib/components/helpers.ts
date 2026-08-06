import type { TauRpcResult } from "../bindings.ts";

export function strip<T, E>(response: TauRpcResult<T, E>) {
  if (response.status == "ok") {
    return response.data;
  } else {
    throw Error(String(response.error));
  }
}
