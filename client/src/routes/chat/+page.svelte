<script>
	import ChatLayout from '$lib/components/chat/ChatLayout.svelte';
	import NewChatModal from '$lib/components/modals/NewChatModal.svelte';
	import ProfileSettingsModal from '$lib/components/modals/ProfileSettingsModal.svelte';
</script>

<svelte:head>
	<title>Bagus Chat</title>
</svelte:head>

<ChatLayout />
<NewChatModal />
<ProfileSettingsModal />