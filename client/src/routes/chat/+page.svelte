<script>
  import { goto } from "$app/navigation";
  import ChatLayout from "$lib/components/chat/ChatLayout.svelte";
  import NewChatModal from "$lib/components/modals/NewChatModal.svelte";
  import ProfileSettingsModal from "$lib/components/modals/ProfileSettingsModal.svelte";
  import { createCurrentUserQueryOptions } from "$lib/queries/user";
  import { createQuery } from "@tanstack/svelte-query";

  const userQueryOptions = createCurrentUserQueryOptions();
  const user = createQuery(() => $userQueryOptions);

  $effect(() => {
    if (!user.data) {
      goto("/login");
    }
  });
</script>

<svelte:head>
  <title>Bagus Chat</title>
</svelte:head>

<ChatLayout />
<NewChatModal />
<ProfileSettingsModal />
