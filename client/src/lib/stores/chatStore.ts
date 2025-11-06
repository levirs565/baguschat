import { writable, derived } from 'svelte/store';

export interface Contact {
	id: number;
	name: string;
	avatar: string;
	lastMessage: string;
}

export interface Message {
	id: number;
	text: string;
	sender: 'me' | string;
	time: string;
	file?: any;
}

type AllMessages = {
	[contactId: number]: Message[];
};

const mockContacts: Contact[] = [
	{ id: 1, name: 'Alice', avatar: 'https://via.placeholder.com/150/FF0000/FFFFFF?text=A', lastMessage: 'OK, sampai jumpa!' },
	{ id: 2, name: 'Bob', avatar: 'https://via.placeholder.com/150/00FF00/FFFFFF?text=B', lastMessage: 'Kamu di mana?' },
	{ id: 3, name: 'Charlie', avatar: 'https://via.placeholder.com/150/0000FF/FFFFFF?text=C', lastMessage: 'Jangan lupa...' }
];

const mockMessages: AllMessages = {
	1: [
		{ id: 1, text: 'Hei, apa kabar?', sender: 'alice', time: '10:30' },
		{ id:2, text: 'Baik! Kamu?', sender: 'me', time: '10:31' },
		{ id: 3, text: 'OK, sampai jumpa!', sender: 'alice', time: '10:32' }
	],
	2: [{ id: 1, text: 'Kamu di mana?', sender: 'bob', time: '11:00' }],
    3: [{ id: 1, text: 'Jangan lupa...', sender: 'charlie', time: '15:20' }]
};

export const contacts = writable<Contact[]>(mockContacts);
const allMessages = writable<AllMessages>(mockMessages);
const activeContactId = writable<number | null>(null);

export const activeContact = derived(
	[contacts, activeContactId],
	([$contacts, $activeContactId]): Contact | null => {
		if (!$activeContactId) return null;
		return $contacts.find(c => c.id === $activeContactId) || null;
	}
);

export const activeMessages = derived(
	[allMessages, activeContactId],
	([$allMessages, $activeContactId]): Message[] => {
		if (!$activeContactId) return []; 
		return $allMessages[$activeContactId] || [];
	}
);

export function selectContact(id: number) {
	activeContactId.set(id);
}
