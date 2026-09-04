<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import * as api from "../api";
import { Download, X, Loader2, Check } from "lucide-vue-next";
import type { UnlistenFn } from "@tauri-apps/api/event";

const show = ref(false);
const updateInfo = ref<api.UpdateInfo | null>(null);
const installing = ref(false);
const extracting = ref(false);
const progress = ref(0);
const dismissed = ref(false);

let unlistenAvailable: UnlistenFn | null = null;
let unlistenProgress: UnlistenFn | null = null;
let unlistenExtracting: UnlistenFn | null = null;

onMounted(async () => {
  unlistenAvailable = await api.onUpdateAvailable((info) => {
    updateInfo.value = info;
    if (!dismissed.value) show.value = true;
  });

  unlistenProgress = await api.onUpdateProgress((e) => {
    if (e.content_length && e.content_length > 0) {
      progress.value = Math.min(100, (e.chunk_length / e.content_length) * 100);
    }
  });

  unlistenExtracting = await api.onUpdateExtracting(() => {
    extracting.value = true;
  });
});

onUnmounted(() => {
  unlistenAvailable?.();
  unlistenProgress?.();
  unlistenExtracting?.();
});

function dismiss() {
  show.value = false;
  dismissed.value = true;
}

async function install() {
  installing.value = true;
  progress.value = 0;
  try {
    await api.installUpdate();
    // App will restart — this code may not execute
  } catch (e) {
    console.error("Update failed:", e);
    installing.value = false;
  }
}
</script>

<template>
  <Transition
    enter-active-class="transition-all duration-220 ease-grok"
    enter-from-class="opacity-0 translate-y-2"
    leave-active-class="transition-all duration-140 ease-grok"
    leave-to-class="opacity-0 translate-y-2"
  >
    <div
      v-if="show && updateInfo"
      class="absolute bottom-4 right-4 z-40 w-[360px] rounded-lg border border-border bg-popover text-popover-foreground shadow-dialog"
    >
      <div class="flex items-start gap-3 p-3.5">
        <div class="flex h-8 w-8 items-center justify-center rounded-md bg-primary/10 shrink-0">
          <Download class="size-4 text-primary" :stroke-width="1.75" />
        </div>
        <div class="flex-1 min-w-0">
          <div class="text-[13px] font-medium">
            Update available — v{{ updateInfo.version }}
          </div>
          <div class="text-[11px] text-muted-foreground mt-0.5">
            Current: v{{ updateInfo.current_version }}
          </div>
          <div v-if="updateInfo.body" class="mt-2 text-[12px] text-muted-foreground max-h-[80px] overflow-y-auto whitespace-pre-wrap">
            {{ updateInfo.body }}
          </div>

          <!-- Progress bar -->
          <div v-if="installing" class="mt-2.5">
            <div class="h-1 w-full rounded-full bg-muted overflow-hidden">
              <div
                class="h-full bg-primary transition-all duration-140 ease-grok"
                :style="{ width: extracting ? '100%' : `${progress}%` }"
              />
            </div>
            <div class="text-[11px] text-muted-foreground mt-1">
              {{ extracting ? "Extracting..." : `Downloading... ${Math.round(progress)}%` }}
            </div>
          </div>

          <!-- Actions -->
          <div v-else class="flex items-center gap-2 mt-2.5">
            <button
              class="inline-flex h-7 items-center rounded-md bg-primary px-2.5 text-[12px] font-medium text-primary-foreground hover:bg-primary/90 transition-colors duration-100"
              @click="install"
            >
              <Download class="size-3 mr-1" :stroke-width="1.75" />
              Download & Restart
            </button>
            <button
              class="inline-flex h-7 items-center rounded-md px-2.5 text-[12px] text-muted-foreground hover:bg-muted transition-colors duration-100"
              @click="dismiss"
            >
              Later
            </button>
          </div>
        </div>
        <button
          v-if="!installing"
          class="flex h-6 w-6 items-center justify-center rounded text-muted-foreground hover:bg-muted transition-colors duration-100 shrink-0"
          aria-label="Dismiss"
          @click="dismiss"
        >
          <X class="size-3.5" :stroke-width="1.75" />
        </button>
        <Loader2 v-else class="size-4 text-muted-foreground animate-spin shrink-0" :stroke-width="1.75" />
      </div>
    </div>
  </Transition>
</template>
