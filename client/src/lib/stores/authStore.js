import { writable } from 'svelte/store';
import { browser } from '$app/environment';

const initialUser = browser ? localStorage.getItem('currentUser') : null;

function createAuthStore() {
	const { subscribe, set } = writable({
		user: initialUser ? JSON.parse(initialUser) : null
	});

	return {
		subscribe,
        /**
        * @param {string} username
        */
		login: (username) => {
			const user = { username, name: username };
			if (browser) localStorage.setItem('currentUser', JSON.stringify(user));
			set({ user });
		},
		logout: () => {
			if (browser) localStorage.removeItem('currentUser');
			set({ user: null });
		}
	};
}
export const authStore = createAuthStore();