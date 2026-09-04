<script setup lang="ts">
import {
  Server,
  Terminal,
  KeyRound,
  FolderOpen,
  HardDrive,
  ShieldCheck,
  Vault,
  Search,
  Plus,
  Settings,
  CircleDot,
  UserCircle,
  ArrowUpCircle,
  PanelLeftClose,
  PanelLeftOpen,
  X,
} from "lucide-vue-next";
import { cn } from "../lib/cn";
import { useUpdateStore } from "../stores/update";
import { useUiStore } from "../stores/ui";

defineProps<{
  activeView: string;
  vaultUnlocked: boolean;
  tabCount: number;
}>();

const emit = defineEmits<{
  navigate: [string];
  "open-command-palette": [];
}>();

const update = useUpdateStore();
const ui = useUiStore();

const navItems = [
  { id: "hosts", label: "Hosts", icon: Server },
  { id: "identities", label: "Identities", icon: UserCircle },
  { id: "terminal", label: "Terminal", icon: Terminal },
  { id: "files", label: "Files", icon: HardDrive },
  { id: "keys", label: "Keys", icon: KeyRound },
  { id: "workspaces", label: "Workspaces", icon: FolderOpen },
  { id: "known-hosts", label: "Known Hosts", icon: ShieldCheck },
  { id: "vault", label: "Vault", icon: Vault },
];

function navigate(id: string) {
  emit("navigate", id);
  ui.closeMobileSidebar();
}
</script>

<template>
  <Transition
    enter-active-class="transition-opacity duration-140"
    enter-from-class="opacity-0"
    leave-active-class="transition-opacity duration-100"
    leave-to-class="opacity-0"
  >
    <div
      v-if="ui.mobileSidebarOpen"
      class="fixed inset-0 z-40 bg-black/50 md:hidden"
      @click="ui.closeMobileSidebar()"
    />
  </Transition>

  <aside
    class="flex flex-col bg-sidebar border-r border-sidebar-border transition-all duration-200 ease-grok md:relative md:translate-x-0"
    :class="cn(
      'z-50',
      ui.sidebarCollapsed ? 'w-[52px] min-w-[52px]' : 'w-[252px] min-w-[196px]',
      'fixed inset-y-0 left-0 md:fixed md:inset-y-auto md:left-auto',
      ui.mobileSidebarOpen ? 'translate-x-0' : '-translate-x-full md:translate-x-0',
    )"
  >
    <div class="flex h-11 items-center gap-2 border-b border-sidebar-border" :class="ui.sidebarCollapsed ? 'justify-center px-1' : 'px-3'">
      <span v-if="!ui.sidebarCollapsed" class="text-[13px] font-semibold tracking-tight text-sidebar-foreground">OpenTermius</span>
      <Server v-else class="size-4 text-sidebar-foreground" :stroke-width="1.75" />
      <button
        class="ml-auto flex h-7 w-7 items-center justify-center rounded-md text-muted-foreground hover:bg-sidebar-accent md:hidden"
        @click="ui.closeMobileSidebar()"
      >
        <X class="size-4" :stroke-width="1.75" />
      </button>
      <button
        v-if="!ui.sidebarCollapsed"
        class="ml-auto hidden md:flex h-6 w-6 items-center justify-center rounded text-muted-foreground hover:bg-sidebar-accent transition-colors duration-100"
        title="Collapse sidebar"
        @click="ui.toggleSidebar()"
      >
        <PanelLeftClose class="size-3.5" :stroke-width="1.75" />
      </button>
    </div>

    <div class="flex flex-col gap-0.5 px-2 pt-2" :class="ui.sidebarCollapsed ? 'px-1.5' : ''">
      <button
        class="flex h-9 items-center gap-2 rounded-md text-[13px] text-sidebar-foreground transition-colors duration-100 hover:bg-sidebar-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
        :class="ui.sidebarCollapsed ? 'justify-center px-1' : 'px-2'"
        :title="ui.sidebarCollapsed ? 'Search (⌘K)' : ''"
        @click="emit('open-command-palette')"
      >
        <Search class="size-3.5 text-muted-foreground shrink-0" :stroke-width="1.75" />
        <template v-if="!ui.sidebarCollapsed">
          <span class="flex-1 text-left">Search...</span>
          <kbd class="text-[10px] text-muted-foreground rounded border border-border px-1 py-0.5">⌘K</kbd>
        </template>
      </button>
      <button
        class="flex h-9 items-center gap-2 rounded-md text-[13px] text-sidebar-foreground transition-colors duration-100 hover:bg-sidebar-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
        :class="ui.sidebarCollapsed ? 'justify-center px-1' : 'px-2'"
        :title="ui.sidebarCollapsed ? 'New Terminal (⌘N)' : ''"
        @click="navigate('terminal')"
      >
        <Plus class="size-3.5 text-muted-foreground shrink-0" :stroke-width="1.75" />
        <template v-if="!ui.sidebarCollapsed">
          <span class="flex-1 text-left">New Terminal</span>
          <kbd class="text-[10px] text-muted-foreground rounded border border-border px-1 py-0.5">⌘N</kbd>
        </template>
      </button>
    </div>

    <nav class="flex flex-col gap-0.5 px-2 pt-3" :class="ui.sidebarCollapsed ? 'px-1.5' : ''">
      <button
        v-for="item in navItems"
        :key="item.id"
        class="group flex h-9 w-full items-center gap-2 rounded-md text-[13px] text-sidebar-foreground transition-colors duration-100 hover:bg-sidebar-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
        :class="cn(
          ui.sidebarCollapsed ? 'justify-center px-1' : 'px-2',
          { 'bg-sidebar-accent text-sidebar-accent-foreground': activeView === item.id },
        )"
        :data-active="activeView === item.id || undefined"
        :title="ui.sidebarCollapsed ? item.label : ''"
        @click="navigate(item.id)"
      >
        <component
          :is="item.icon"
          class="size-3.5 shrink-0 text-muted-foreground group-data-[active]:text-sidebar-accent-foreground"
          :stroke-width="1.75"
        />
        <template v-if="!ui.sidebarCollapsed">
          <span class="min-w-0 flex-1 truncate text-left">{{ item.label }}</span>
          <span
            v-if="item.id === 'terminal' && tabCount > 0"
            class="text-[10px] font-medium rounded bg-muted px-1.5 py-0.5 text-muted-foreground"
          >
            {{ tabCount }}
          </span>
        </template>
        <span
          v-if="ui.sidebarCollapsed && item.id === 'terminal' && tabCount > 0"
          class="absolute top-1 right-1 size-1.5 rounded-full bg-primary"
        />
      </button>
    </nav>

    <div class="flex-1"></div>

    <div v-if="ui.sidebarCollapsed" class="px-1.5 pb-1.5">
      <button
        class="flex h-8 w-full items-center justify-center rounded-md text-muted-foreground hover:bg-sidebar-accent transition-colors duration-100"
        title="Expand sidebar"
        @click="ui.toggleSidebar()"
      >
        <PanelLeftOpen class="size-3.5" :stroke-width="1.75" />
      </button>
    </div>

    <div class="border-t border-sidebar-border px-2 py-2 flex flex-col gap-1" :class="ui.sidebarCollapsed ? 'px-1.5' : ''">
      <button
        class="flex h-10 w-full items-center gap-2 rounded-md text-[13px] text-sidebar-foreground transition-colors duration-100 hover:bg-sidebar-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
        :class="cn(
          ui.sidebarCollapsed ? 'justify-center px-1' : 'px-2',
          { 'bg-sidebar-accent text-sidebar-accent-foreground': activeView === 'vault' },
        )"
        :title="ui.sidebarCollapsed ? `Vault (${vaultUnlocked ? 'Unlocked' : 'Locked'})` : ''"
        @click="navigate('vault')"
      >
        <div class="relative shrink-0">
          <Vault class="size-4 text-muted-foreground" :stroke-width="1.75" />
          <CircleDot
            class="absolute -right-0.5 -top-0.5 size-2"
            :class="vaultUnlocked ? 'text-green-500' : 'text-muted-foreground'"
            :stroke-width="0"
            fill="currentColor"
          />
        </div>
        <template v-if="!ui.sidebarCollapsed">
          <div class="flex flex-col items-start min-w-0">
            <span class="text-[12px] font-medium truncate">Vault</span>
            <span class="text-[11px] text-muted-foreground truncate">
              {{ vaultUnlocked ? "Unlocked" : "Locked" }}
            </span>
          </div>
        </template>
      </button>
      <button
        class="flex h-9 w-full items-center gap-2 rounded-md text-[13px] text-sidebar-foreground transition-colors duration-100 hover:bg-sidebar-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
        :class="cn(
          ui.sidebarCollapsed ? 'justify-center px-1' : 'px-2',
          { 'bg-sidebar-accent text-sidebar-accent-foreground': activeView === 'settings' },
        )"
        :title="ui.sidebarCollapsed ? 'Settings' : ''"
        @click="navigate('settings')"
      >
        <Settings class="size-3.5 text-muted-foreground shrink-0" :stroke-width="1.75" />
        <template v-if="!ui.sidebarCollapsed">
          <span class="flex-1 text-left">Settings</span>
          <span
            v-if="update.available"
            class="flex h-4 items-center gap-0.5 rounded-full bg-primary px-1.5 text-[10px] font-medium text-primary-foreground"
          >
            <ArrowUpCircle class="size-2.5" :stroke-width="2" />
            {{ update.version }}
          </span>
        </template>
        <span
          v-if="ui.sidebarCollapsed && update.available"
          class="absolute top-1 right-1 size-1.5 rounded-full bg-primary"
        />
      </button>
    </div>
  </aside>
</template>
