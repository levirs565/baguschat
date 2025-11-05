<script>
	import { Listgroup } from 'flowbite-svelte';
	import { contacts } from '$lib/stores/chatStore.js';
	import ContactItem from './ContactItem.svelte';
	import ProfileButton from '../common/ProfileButton.svelte';
	import SettingsButton from '../common/SettingsButton.svelte';
	import NewChatButton from '../common/NewChatButton.svelte';
</script>

<div class="flex flex-col h-full">
	<div class="p-4 border-b dark:border-gray-700 flex justify-between items-center">
		<ProfileButton />
		
		<div class="flex space-x-2">
			<NewChatButton /> <SettingsButton />
		</div>
	</div>

	<div class="flex-1 overflow-y-auto">
		<Listgroup class="border-none">
			{#each $contacts as contact (contact.id)}
				<ContactItem {contact} />
			{/each}
		</Listgroup>
	</div>
</div>