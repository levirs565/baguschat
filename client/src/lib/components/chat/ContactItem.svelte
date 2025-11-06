<script lang="ts">
	import { Avatar, Heading } from 'flowbite-svelte';
	import { selectContact, activeContact } from '$lib/stores/chatStore';
	import type { Contact } from '$lib/stores/chatStore';

	export let contact: Contact;

	$: isActive = $activeContact && $activeContact.id === contact.id;
</script>

<button
	type="button"
	on:click={() => selectContact(contact.id)}
	class="flex items-center w-full px-4 py-2 text-left font-medium border-x-0 border-t-0 border-b dark:border-gray-700 cursor-pointer"
	
	class:bg-blue-700={isActive}
	class:text-white={isActive}
	class:dark:bg-gray-600={isActive}
	
	class:bg-white={!isActive}
	class:text-gray-900={!isActive}
	class:dark:bg-gray-800={!isActive}
	class:dark:text-white={!isActive}
	class:hover:bg-gray-100={!isActive}
	class:dark:hover:bg-gray-600={!isActive}
>
	<div class="flex items-center space-x-4">
		<Avatar src={contact.avatar} />
		<div class="flex-1 min-w-0">
			<Heading tag="h5" class="truncate">{contact.name}</Heading>
			<p class="text-sm text-gray-500 dark:text-gray-400 truncate">
				{contact.lastMessage}
			</p>
		</div>
	</div>
</button>