import { writable } from 'svelte/store';

const mockContacts = [
	{ id: 1, name: 'Alice', avatar: 'https://via.placeholder.com/150/FF0000/FFFFFF?text=A', lastMessage: 'OK, sampai jumpa!' },
	{ id: 2, name: 'Bob', avatar: 'https://via.placeholder.com/150/00FF00/FFFFFF?text=B', lastMessage: 'Kamu di mana?' },
	{ id: 3, name: 'Charlie', avatar: 'https://via.placeholder.com/150/0000FF/FFFFFF?text=C', lastMessage: 'Jangan lupa...' }
];

export const contacts = writable(mockContacts);