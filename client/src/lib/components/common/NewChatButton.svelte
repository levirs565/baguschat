<script>
	import UserAddOutline from 'flowbite-svelte-icons/UserAddOutline.svelte';
	import { isNewChatModalOpen } from '$lib/stores/uiStore.js';
</script>

<button
	type="button"
	on:click={() => isNewChatModalOpen.set(true)}
	aria-label="Mulai chat baru"
	class="p-2 cursor-pointer text-gray-900 bg-white border border-gray-200 hover:bg-gray-100 focus:ring-4 focus:ring-gray-200 rounded-lg dark:bg-gray-800 dark:text-gray-400 dark:border-gray-600 dark:hover:bg-gray-700 dark:focus:ring-gray-700"
>
	<UserAddOutline class="w-6 h-6" />
</button>