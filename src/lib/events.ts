import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export async function typedListen<T>(
  event: string,
  handler: (payload: T) => void,
): Promise<UnlistenFn> {
  return listen<T>(event, (e) => handler(e.payload));
}
