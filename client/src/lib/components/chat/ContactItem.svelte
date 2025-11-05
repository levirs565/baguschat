<script>
	import { ListgroupItem, Avatar, Heading, P } from 'flowbite-svelte';
	export let contact;
</script>

<ListgroupItem class="border-x-0 border-t-0 border-b dark:border-gray-700 cursor-pointer">
	<div class="flex items-center space-x-4">
		<Avatar src={contact.avatar} />
		<div class="flex-1 min-w-0">
			<Heading tag="h5" class="truncate">{contact.name}</Heading>
			<p class="text-sm text-gray-500 dark:text-gray-400 truncate">{contact.lastMessage}</p>
		</div>
	</div>
</ListgroupItem>