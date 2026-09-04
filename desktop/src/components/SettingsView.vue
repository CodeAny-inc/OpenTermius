<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from "vue";
import * as api from "../api";
import Button from "./ui/Button.vue";
import Badge from "./ui/Badge.vue";
import {
  Settings as SettingsIcon,
  Download,
  RefreshCw,
  CheckCircle2,
  ArrowUpCircle,
  Loader2,
  AlertCircle,
  Github,
  ExternalLink,
  Info,
} from "lucide-vue-next";
import type { UnlistenFn } from "@tauri-apps/api/event";

const appInfo = ref<api.AppInfo | null>(null);
const updateInfo = ref<api.UpdateInfo | null>(null);
const checking = ref(false);
const installing = ref(false);
const extracting = ref(false);
const progress = ref(0);
const error = ref("");
const lastChecked = ref<Date | null>(null);

let unlistenProgress: UnlistenFn | null = null;
let unlistenExtracting: UnlistenFn | null = null;

const updateAvailable = computed(
  () => updateInfo.value?.available === true,
);

const statusText = computed(() => {
  if (installing.value) return "Installing...";
  if (extracting.value) return "Extracting...";
  if (checking.value) return "Checking for updates...";
  if (error.value) return "Update check failed";
  if (updateAvailable.value) return "Update available";
  if (updateInfo.value && !updateInfo.value.available) return "Up to date";
  return "Not checked yet";
});

onMounted(async () => {
  // Load app info immediately
  try {
    appInfo.value = await api.getAppInfo();
  } catch (e) {
    console.error("Failed to get app info:", e);
  }

  // Register progress listeners
  try {
    unlistenProgress = await api.onUpdateProgress((e) => {
      if (e.content_length && e.content_length > 0) {
        progress.value = Math.min(100, (e.chunk_length / e.content_length) * 100);
      }
    });
  } catch (e) {
    console.error("listen progress failed:", e);
  }

  try {
    unlistenExtracting = await api.onUpdateExtracting(() => {
      extracting.value = true;
    });
  } catch (e) {
    console.error("listen extracting failed:", e);
  }

  // Auto-check on mount
  await checkForUpdates();
});

onUnmounted(() => {
  unlistenProgress?.();
  unlistenExtracting?.();
});

async function checkForUpdates() {
  if (checking.value) return;
  checking.value = true;
  error.value = "";
  try {
    const info = await api.checkForUpdates();
    updateInfo.value = info;
    lastChecked.value = new Date();
  } catch (e) {
    error.value = String(e);
    console.error("Update check failed:", e);
  } finally {
    checking.value = false;
  }
}

async function downloadAndInstall() {
  if (installing.value) return;
  installing.value = true;
  progress.value = 0;
  extracting.value = false;
  error.value = "";
  try {
    await api.installUpdate();
    // App will restart — this code may not execute
  } catch (e) {
    error.value = String(e);
    console.error("Update install failed:", e);
    installing.value = false;
  }
}

function openGitHub() {
  // Use the shell plugin to open the URL in the default browser
  import("@tauri-apps/plugin-shell").then(({ open }) => {
    open("https://github.com/CodeAny-inc/OpenTermius");
  }).catch(() => {
    window.open("https://github.com/CodeAny-inc/OpenTermius", "_blank");
  });
}

function openReleases() {
  import("@tauri-apps/plugin-shell").then(({ open }) => {
    open("https://github.com/CodeAny-inc/OpenTermius/releases");
  }).catch(() => {
    window.open("https://github.com/CodeAny-inc/OpenTermius/releases", "_blank");
  });
}
</script>

<template>
  <div class="flex flex-col h-full overflow-hidden">
    <!-- Header -->
    <div class="flex h-11 items-center gap-2 border-b border-border px-4">
      <SettingsIcon class="size-4 text-muted-foreground" :stroke-width="1.75" />
      <h2 class="text-[14px] font-semibold">Settings</h2>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto p-6">
      <div class="max-w-[640px] mx-auto flex flex-col gap-6">

        <!-- About section -->
        <section class="rounded-lg border border-border bg-card p-5">
          <h3 class="text-[14px] font-semibold mb-4 flex items-center gap-2">
            <Info class="size-4 text-muted-foreground" :stroke-width="1.75" />
            About
          </h3>
          <div class="flex items-center gap-4 mb-4">
            <div class="flex h-14 w-14 items-center justify-center rounded-xl bg-primary/10 shrink-0">
              <SettingsIcon class="size-7 text-primary" :stroke-width="1.5" />
            </div>
            <div class="flex-1 min-w-0">
              <div class="text-[16px] font-semibold">{{ appInfo?.name ?? 'OpenTermius' }}</div>
              <div class="text-[12px] text-muted-foreground mt-0.5">
                Version <span class="font-mono font-medium text-foreground">v{{ appInfo?.version ?? '—' }}</span>
              </div>
            </div>
            <Badge>{{ appInfo?.platform ?? '—' }} / {{ appInfo?.arch ?? '—' }}</Badge>
          </div>
          <div class="flex items-center gap-2 pt-3 border-t border-border">
            <button
              class="inline-flex h-7 items-center gap-1.5 rounded-md px-2.5 text-[12px] text-muted-foreground hover:bg-muted transition-colors duration-100"
              @click="openGitHub"
            >
              <Github class="size-3.5" :stroke-width="1.75" />
              GitHub
              <ExternalLink class="size-3" :stroke-width="1.75" />
            </button>
            <button
              class="inline-flex h-7 items-center gap-1.5 rounded-md px-2.5 text-[12px] text-muted-foreground hover:bg-muted transition-colors duration-100"
              @click="openReleases"
            >
              <Download class="size-3.5" :stroke-width="1.75" />
              Releases
              <ExternalLink class="size-3" :stroke-width="1.75" />
            </button>
          </div>
        </section>

        <!-- Updates section -->
        <section class="rounded-lg border border-border bg-card p-5">
          <h3 class="text-[14px] font-semibold mb-4 flex items-center gap-2">
            <ArrowUpCircle class="size-4 text-muted-foreground" :stroke-width="1.75" />
            Updates
          </h3>

          <!-- Status row -->
          <div class="flex items-center gap-3 mb-4">
            <!-- Status icon -->
            <div class="flex h-9 w-9 items-center justify-center rounded-md shrink-0"
              :class="{
                'bg-green-500/10': updateInfo && !updateInfo.available && !error,
                'bg-primary/10': updateAvailable,
                'bg-destructive/10': error,
                'bg-muted': checking || (!updateInfo && !error),
              }"
            >
              <CheckCircle2 v-if="updateInfo && !updateInfo.available && !error" class="size-5 text-green-500" :stroke-width="1.75" />
              <ArrowUpCircle v-else-if="updateAvailable" class="size-5 text-primary" :stroke-width="1.75" />
              <AlertCircle v-else-if="error" class="size-5 text-destructive" :stroke-width="1.75" />
              <Loader2 v-else-if="checking" class="size-5 text-muted-foreground animate-spin" :stroke-width="1.75" />
              <Info v-else class="size-5 text-muted-foreground" :stroke-width="1.75" />
            </div>

            <div class="flex-1 min-w-0">
              <div class="text-[13px] font-medium">{{ statusText }}</div>
              <div class="text-[11px] text-muted-foreground mt-0.5">
                <template v-if="updateAvailable">
                  v{{ updateInfo?.version }} is available (current: v{{ updateInfo?.current_version }})
                </template>
                <template v-else-if="updateInfo && !updateInfo.available">
                  You're on the latest version
                </template>
                <template v-else-if="error">
                  {{ error }}
                </template>
                <template v-else>
                  Click "Check Now" to check for updates
                </template>
              </div>
              <div v-if="lastChecked" class="text-[10px] text-muted-foreground mt-0.5">
                Last checked: {{ lastChecked.toLocaleTimeString() }}
              </div>
            </div>

            <!-- Check Now button -->
            <Button
              size="sm"
              variant="outline"
              :disabled="checking || installing"
              @click="checkForUpdates"
            >
              <RefreshCw class="size-3.5 mr-1" :class="{ 'animate-spin': checking }" :stroke-width="1.75" />
              Check Now
            </Button>
          </div>

          <!-- Update details + install -->
          <div v-if="updateAvailable" class="border-t border-border pt-4">
            <!-- Release notes -->
            <div v-if="updateInfo?.body" class="mb-4">
              <div class="text-[11px] font-medium text-muted-foreground uppercase tracking-wide mb-1.5">
                Release Notes
              </div>
              <div class="text-[12px] text-muted-foreground whitespace-pre-wrap rounded-md bg-muted/50 p-3 max-h-[200px] overflow-y-auto">
                {{ updateInfo.body }}
              </div>
            </div>

            <!-- Download progress -->
            <div v-if="installing" class="mb-4">
              <div class="h-1.5 w-full rounded-full bg-muted overflow-hidden">
                <div
                  class="h-full bg-primary transition-all duration-140 ease-grok"
                  :style="{ width: extracting ? '100%' : `${progress}%` }"
                />
              </div>
              <div class="text-[11px] text-muted-foreground mt-1.5 flex items-center gap-1.5">
                <Loader2 class="size-3 animate-spin" :stroke-width="1.75" />
                {{ extracting ? "Extracting and installing..." : `Downloading... ${Math.round(progress)}%` }}
              </div>
            </div>

            <!-- Install button -->
            <div v-else class="flex items-center gap-2">
              <Button size="sm" @click="downloadAndInstall">
                <Download class="size-3.5 mr-1" :stroke-width="1.75" />
                Download & Restart
              </Button>
              <span class="text-[11px] text-muted-foreground">
                The app will restart automatically after installation
              </span>
            </div>
          </div>

          <!-- Error retry -->
          <div v-if="error && !installing" class="border-t border-border pt-4">
            <Button size="sm" variant="outline" @click="checkForUpdates">
              <RefreshCw class="size-3.5 mr-1" :stroke-width="1.75" />
              Try Again
            </Button>
          </div>
        </section>

        <!-- Auto-update info -->
        <section class="rounded-lg border border-border bg-muted/30 p-4">
          <div class="flex items-start gap-3">
            <Info class="size-4 text-muted-foreground shrink-0 mt-0.5" :stroke-width="1.75" />
            <div class="text-[12px] text-muted-foreground leading-relaxed">
              <p class="font-medium text-foreground mb-1">Automatic Updates</p>
              <p>
                OpenTermius automatically checks for updates on startup.
                When a new version is available, a notification banner appears
                in the bottom-right corner. You can also manually check for
                updates here at any time.
              </p>
              <p class="mt-2">
                Updates are downloaded, signature-verified, and installed
                automatically. The app restarts to apply the update.
              </p>
            </div>
          </div>
        </section>

      </div>
    </div>
  </div>
</template>
