import {  derived, get, writable } from "svelte/store";
import { client, ClientSession } from "$lib/api";

function createSessionStore() {
  const store = writable<ClientSession | null>(null);
  const { set, subscribe } = store;

  client.createClientSession(undefined).then((session) => {
    set(session);
  }).catch((e) => {
    console.log(e);
  })

  return {
    subscribe,
    async login(username: string, password: string) {
      set(await client.login(username, password));
    },
    async logout() {
      const client = get(store);
      await client?.logout();
      set(null);
    },
  };
}

export const sessionStore = createSessionStore();

export const currentUserId = derived([sessionStore], ([session]) => session?.userId)