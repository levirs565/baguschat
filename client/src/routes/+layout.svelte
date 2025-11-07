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
  import { createQuery, QueryClientProvider } from "@tanstack/svelte-query";


  let { children } = $props();

	import { queryClient } from "$lib/queries/core.js";
  
</script>

<svelte:head>
  <link rel="icon" href={favicon} />
</svelte:head>

<QueryClientProvider client={queryClient}>
{@render children()}
</QueryClientProvider>
