<script>
	import ChatSidebar from '$lib/components/chat/ChatSidebar.svelte';
	import ChatWindow from '$lib/components/chat/ChatWindow.svelte';
</script>

<div class="flex h-screen w-full bg-white dark:bg-gray-800">
	<div class="w-full md:w-1/3 lg:w-1/4 h-screen border-r dark:border-gray-700">
		<ChatSidebar />
	</div>

	<div class="hidden md:flex flex-col flex-1 h-screen">
		<ChatWindow />
	</div>
</div>