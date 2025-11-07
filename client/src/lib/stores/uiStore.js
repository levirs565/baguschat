import { writable } from 'svelte/store';

export const isNewChatModalOpen = writable(false);
export const isProfileModalOpen = writable(false);
export const isSecretChatModalOpen = writable(false)