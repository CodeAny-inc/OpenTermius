<script setup lang="ts">
import {
  Server,
  Terminal,
  KeyRound,
  FolderOpen,
  ShieldCheck,
  Vault,
  Search,
  Plus,
  Settings,
  CircleDot,
  UserCircle,
} from "lucide-vue-next";
import { cn } from "../lib/cn";

defineProps<{
  activeView: string;
  vaultUnlocked: boolean;
  tabCount: number;
}>();

const emit = defineEmits<{
  navigate: [string];
  "open-command-palette": [];
}>();

const navItems = [
  { id: "hosts", label: "Hosts", icon: Server },
  { id: "identities", label: "Identities", icon: UserCircle },
  { id: "terminal", label: "Terminal", icon: Terminal },
  { id: "keys", label: "Keys", icon: KeyRound },
  { id: "workspaces", label: "Workspaces", icon: FolderOpen },
  { id: "known-hosts", label: "Known Hosts", icon: ShieldCheck },
  { id: "vault", label: "Vault", icon: Vault },
];
</script>

<template>
  <aside class="flex w-[252px] min-w-[196px] flex-col bg-sidebar border-r border-sidebar-border">
    <!-- Header -->
    <div class="flex h-11 items-center gap-2 px-3 border-b border-sidebar-border">
      <span class="text-[13px] font-semibold tracking-tight text-sidebar-foreground">OpenTermius</span>
    </div>

    <!-- Quick actions -->
    <div class="flex flex-col gap-0.5 px-2 pt-2">
      <button
        class="flex h-9 items-center gap-2 rounded-md px-2 text-[13px] text-sidebar-foreground transition-colors duration-100 hover:bg-sidebar-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
        @click="emit('open-command-palette')"
      >
        <Search class="size-3.5 text-muted-foreground" :stroke-width="1.75" />
        <span class="flex-1 text-left">Search...</span>
        <kbd class="text-[10px] text-muted-foreground rounded border border-border px-1 py-0.5">⌘K</kbd>
      </button>
      <button
        class="flex h-9 items-center gap-2 rounded-md px-2 text-[13px] text-sidebar-foreground transition-colors duration-100 hover:bg-sidebar-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
        @click="emit('navigate', 'terminal')"
      >
        <Plus class="size-3.5 text-muted-foreground" :stroke-width="1.75" />
        <span class="flex-1 text-left">New Terminal</span>
        <kbd class="text-[10px] text-muted-foreground rounded border border-border px-1 py-0.5">⌘N</kbd>
      </button>
    </div>

    <!-- Navigation -->
    <nav class="flex flex-col gap-0.5 px-2 pt-3">
      <button
        v-for="item in navItems"
        :key="item.id"
        class="group flex h-9 w-full items-center gap-2 rounded-md px-2 text-[13px] text-sidebar-foreground transition-colors duration-100 hover:bg-sidebar-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
        :class="{ 'bg-sidebar-accent text-sidebar-accent-foreground': activeView === item.id }"
        :data-active="activeView === item.id || undefined"
        @click="emit('navigate', item.id)"
      >
        <component
          :is="item.icon"
          class="size-3.5 shrink-0 text-muted-foreground group-data-[active]:text-sidebar-accent-foreground"
          :stroke-width="1.75"
        />
        <span class="min-w-0 flex-1 truncate text-left">{{ item.label }}</span>
        <span
          v-if="item.id === 'terminal' && tabCount > 0"
          class="text-[10px] font-medium rounded bg-muted px-1.5 py-0.5 text-muted-foreground"
        >
          {{ tabCount }}
        </span>
      </button>
    </nav>

    <!-- Spacer -->
    <div class="flex-1"></div>

    <!-- Footer: Vault status + Settings -->
    <div class="border-t border-sidebar-border px-2 py-2 flex flex-col gap-1">
      <button
        class="flex h-10 w-full items-center gap-2 rounded-md px-2 text-[13px] text-sidebar-foreground transition-colors duration-100 hover:bg-sidebar-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
        :class="{ 'bg-sidebar-accent text-sidebar-accent-foreground': activeView === 'vault' }"
        @click="emit('navigate', 'vault')"
      >
        <div class="relative">
          <Vault class="size-4 text-muted-foreground" :stroke-width="1.75" />
          <CircleDot
            class="absolute -right-0.5 -top-0.5 size-2"
            :class="vaultUnlocked ? 'text-green-500' : 'text-muted-foreground'"
            :stroke-width="0"
            fill="currentColor"
          />
        </div>
        <div class="flex flex-col items-start min-w-0">
          <span class="text-[12px] font-medium truncate">Vault</span>
          <span class="text-[11px] text-muted-foreground truncate">
            {{ vaultUnlocked ? "Unlocked" : "Locked" }}
          </span>
        </div>
      </button>
      <button
        class="flex h-9 w-full items-center gap-2 rounded-md px-2 text-[13px] text-sidebar-foreground transition-colors duration-100 hover:bg-sidebar-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
        :class="{ 'bg-sidebar-accent text-sidebar-accent-foreground': activeView === 'settings' }"
        @click="emit('navigate', 'settings')"
      >
        <Settings class="size-3.5 text-muted-foreground" :stroke-width="1.75" />
        <span class="flex-1 text-left">Settings</span>
      </button>
    </div>
  </aside>
</template>
