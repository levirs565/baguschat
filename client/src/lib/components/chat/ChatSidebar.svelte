<script>
  import { Dropdown, DropdownItem, Heading, Listgroup } from "flowbite-svelte";
  import { chatPartnersStore } from "$lib/stores/chatStore";
  import ContactItem from "./ContactItem.svelte";
  import ProfileButton from "../common/ProfileButton.svelte";
  import SettingsButton from "../common/SettingsButton.svelte";
  import NewChatButton from "../common/NewChatButton.svelte";
  import { sessionStore } from "$lib/stores/sessionStore";
  import { createCurrentUserQueryOptions } from "$lib/queries/user";
  import { createQuery } from "@tanstack/svelte-query";

  const userQueryOptions = createCurrentUserQueryOptions();
  const user = createQuery(() => $userQueryOptions);
</script>

<div class="flex flex-col h-full">
  <div
    class="p-4 border-b dark:border-gray-700 flex justify-between items-center"
  >
    <ProfileButton />

    <p class="font-semibold dark:text-white">{user.data?.user?.username}</p>

    <div class="flex space-x-2">
      <NewChatButton />
      <SettingsButton />
      <Dropdown simple>
        <DropdownItem onclick={() => sessionStore.logout()}>Logout</DropdownItem
        >
      </Dropdown>
    </div>
  </div>

  <div class="flex-1 overflow-y-auto">
    <Listgroup class="border-none">
      {#each $chatPartnersStore as contact (contact.id)}
        <ContactItem {contact} />
      {/each}
    </Listgroup>
  </div>
</div>
