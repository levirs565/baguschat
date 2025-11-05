<script>
	import { DarkMode } from 'flowbite-svelte'; 
</script>

<div class="flex items-center justify-center min-h-screen bg-gray-50 dark:bg-gray-900">
	
	<div class="absolute top-4 right-4">
		<DarkMode />
	</div>

	<slot />

</div>