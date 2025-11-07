<script>
	import { Avatar } from 'flowbite-svelte';
	import { isProfileModalOpen } from '$lib/stores/uiStore.js';
</script>

<button
	type="button"
	class="flex items-center cursor-pointer"
	aria-label="Buka pengaturan profil"
>
	<Avatar src="" />
</button>