import { sessionStore } from "$lib/stores/sessionStore";
import { createQuery } from "@tanstack/svelte-query";
import { derived } from "svelte/store";

export function createCurrentUserQueryOptions() {
  return derived([sessionStore], ([session]) => {
    return {
      queryKey: ["user", session?.userId ?? ""],
      queryFn: async () => {
        return (await session?.getState()) ?? null;
      },
    };
  });
}

export function createUserDataQueryOptions(userId: string) {
  return derived([sessionStore], ([session]) => {
    return {
      queryKey: ["user-data", userId],
      queryFn: async () => {
        if (!session) return null;
        return (await session.getUser(userId)) ?? null
      }
    }
  })
}

export function createUserListQueryOptions(username: string) {
  return derived([sessionStore], ([session]) => {
    return {
      queryKey: ["user-list", username],
      queryFn: async () => {
        if (!session) return null;
        return (await session.listUsers(username)) ?? null
      }
    }
  })
}