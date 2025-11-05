<!-- <script lang="ts">
  import "../lib/api.ts";
  import "../app.css";
  import favicon from "$lib/assets/favicon.svg";

  let { children } = $props();

  export const ssr = false;
</script>


{@render children()} -->

<script>
  import "../lib/api.ts";
  import '../app.css';
  import favicon from "$lib/assets/favicon.svg";

  let { children } = $props();

  export const ssr = false;

	import { authStore } from '$lib/stores/authStore.js';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
  
	authStore.subscribe((storeData) => {
    const isAuthRoute = $page.url.pathname.startsWith('/(auth)');
		const isChatRoute = $page.url.pathname === '/chat';
    
		if (!storeData.user && isChatRoute) {
      goto('/login');
		}
    
		if (storeData.user && isAuthRoute) {
      goto('/chat');
		}
	});
</script>

<svelte:head>
  <link rel="icon" href={favicon} />
</svelte:head>

{@render children()}
