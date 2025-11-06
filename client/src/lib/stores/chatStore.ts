import type { ChatWs, DecryptedChat2, DecryptedChatPartner } from "$lib/api";
import { writable, derived, get } from "svelte/store";
import { currentUserId, sessionStore } from "./sessionStore";

export interface Contact {
  id: number;
  name: string;
  avatar: string;
  lastMessage: string;
}

export interface Message {
  id: number;
  text: string;
  sender: "me" | string;
  time: string;
  file?: any;
}

export const chatPartnersStore = writable<DecryptedChatPartner[]>([]);
export const activeContactId = writable<string | null>(null);
export const activeMessages = writable<DecryptedChat2[]>([]);
export const chatWsStore = writable<ChatWs | null>();

const onReceiveChat = (event: CustomEvent<DecryptedChat2>) => {
  const activeId = get(activeContactId);

  if (
    activeId == event.detail.sender_id ||
    activeId == event.detail.receiver_id
  ) {
    activeMessages.update((messages) => [...messages, event.detail]);
  }

  chatPartnersStore.update((partners) => {
    let result = [...partners];
    let index = result.findIndex(
      (partner) =>
        partner.id == event.detail.sender_id ||
        partner.id == event.detail.receiver_id
    );
    if (index >= 0) {
      result[index] = {
        id: result[index].id,
        last_chat: event.detail,
      };
    } else {
      result.push({
        id:
          event.detail.sender_id == get(currentUserId)
            ? event.detail.receiver_id!
            : event.detail.sender_id,
        last_chat: event.detail,
      });
    }

    return result.toSorted(
      (a, b) =>
        new Date(b.last_chat.created_at).getTime() -
        new Date(a.last_chat.created_at).getTime()
    );
  });
};

sessionStore.subscribe(async (session) => {
  chatWsStore.update((chat) => {
    if (chat) {
      chat.removeEventListener("receive-chat", onReceiveChat);
      chat.ws.close();
    }
    return null;
  });

  if (!session) {
    chatPartnersStore.set([]);
    return;
  }

  const chatPartners = await session.getChatPartners();
  chatPartnersStore.set(chatPartners);

  const chatWs = await session.createChatWs();
  chatWs.addEventListener("receive-chat", onReceiveChat);
  chatWsStore.set(chatWs);
});

derived([sessionStore, activeContactId], ([session, activeContactId]) => ({
  session,
  activeContactId,
})).subscribe(async ({ session, activeContactId }) => {
  if (!session) {
    activeMessages.set([]);
    return;
  }

  const chats = await session.getChats();
  activeMessages.set(chats);
});

export function selectContact(id: string) {
  activeContactId.set(id);
}
