<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import * as api from "../api";
import { useUpdateStore } from "../stores/update";
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

const update = useUpdateStore();
const appInfo = ref<api.AppInfo | null>(null);

const updateAvailable = computed(() => update.available);

const statusText = computed(() => {
  if (update.installing) return "Installing...";
  if (update.extracting) return "Extracting...";
  if (update.checking) return "Checking for updates...";
  if (update.error) return "Update check failed";
  if (updateAvailable.value) return "Update available";
  if (update.info && !update.info.available) return "Up to date";
  return "Not checked yet";
});

onMounted(async () => {
  try {
    appInfo.value = await api.getAppInfo();
  } catch (e) {
    console.error("Failed to get app info:", e);
  }
  // Refresh update status if not recently checked
  if (!update.lastChecked) {
    await update.check();
  }
});

function openGitHub() {
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
    <div class="flex h-11 items-center gap-2 border-b border-border px-4 pl-12 md:pl-4">
      <SettingsIcon class="size-4 text-muted-foreground" :stroke-width="1.75" />
      <h2 class="text-[14px] font-semibold">Settings</h2>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto p-4 sm:p-6">
      <div class="max-w-[640px] mx-auto flex flex-col gap-4 sm:gap-6">

        <!-- About section -->
        <section class="rounded-lg border border-border bg-card p-4 sm:p-5">
          <h3 class="text-[14px] font-semibold mb-4 flex items-center gap-2">
            <Info class="size-4 text-muted-foreground" :stroke-width="1.75" />
            About
          </h3>
          <div class="flex items-center gap-3 sm:gap-4 mb-4">
            <div class="flex h-12 w-12 sm:h-14 sm:w-14 items-center justify-center rounded-xl bg-primary/10 shrink-0">
              <SettingsIcon class="size-6 sm:size-7 text-primary" :stroke-width="1.5" />
            </div>
            <div class="flex-1 min-w-0">
              <div class="text-[15px] sm:text-[16px] font-semibold truncate">{{ appInfo?.name ?? 'OpenTermius' }}</div>
              <div class="text-[12px] text-muted-foreground mt-0.5">
                Version <span class="font-mono font-medium text-foreground">v{{ appInfo?.version ?? '—' }}</span>
              </div>
            </div>
            <Badge class="hidden sm:inline-flex">{{ appInfo?.platform ?? '—' }} / {{ appInfo?.arch ?? '—' }}</Badge>
          </div>
          <div class="flex flex-wrap items-center gap-2 pt-3 border-t border-border">
            <button
              class="inline-flex h-7 items-center gap-1.5 rounded-md px-2.5 text-[12px] text-muted-foreground hover:bg-muted transition-colors duration-100"
              @click="openGitHub"
            >
              <Github class="size-3.5" :stroke-width="1.75" />
              <span class="hidden xs:inline">GitHub</span>
              <ExternalLink class="size-3" :stroke-width="1.75" />
            </button>
            <button
              class="inline-flex h-7 items-center gap-1.5 rounded-md px-2.5 text-[12px] text-muted-foreground hover:bg-muted transition-colors duration-100"
              @click="openReleases"
            >
              <Download class="size-3.5" :stroke-width="1.75" />
              <span class="hidden xs:inline">Releases</span>
              <ExternalLink class="size-3" :stroke-width="1.75" />
            </button>
          </div>
        </section>

        <!-- Updates section -->
        <section class="rounded-lg border border-border bg-card p-4 sm:p-5">
          <h3 class="text-[14px] font-semibold mb-4 flex items-center gap-2">
            <ArrowUpCircle class="size-4 text-muted-foreground" :stroke-width="1.75" />
            Updates
          </h3>

          <!-- Status row -->
          <div class="flex items-start sm:items-center gap-3 mb-4 flex-wrap">
            <!-- Status icon -->
            <div class="flex h-9 w-9 items-center justify-center rounded-md shrink-0"
              :class="{
                'bg-green-500/10': update.info && !update.info.available && !update.error,
                'bg-primary/10': updateAvailable,
                'bg-destructive/10': update.error,
                'bg-muted': update.checking || (!update.info && !update.error),
              }"
            >
              <CheckCircle2 v-if="update.info && !update.info.available && !update.error" class="size-5 text-green-500" :stroke-width="1.75" />
              <ArrowUpCircle v-else-if="updateAvailable" class="size-5 text-primary" :stroke-width="1.75" />
              <AlertCircle v-else-if="update.error" class="size-5 text-destructive" :stroke-width="1.75" />
              <Loader2 v-else-if="update.checking" class="size-5 text-muted-foreground animate-spin" :stroke-width="1.75" />
              <Info v-else class="size-5 text-muted-foreground" :stroke-width="1.75" />
            </div>

            <div class="flex-1 min-w-0">
              <div class="text-[13px] font-medium">{{ statusText }}</div>
              <div class="text-[11px] text-muted-foreground mt-0.5">
                <template v-if="updateAvailable">
                  v{{ update.version }} is available (current: v{{ update.currentVersion }})
                </template>
                <template v-else-if="update.info && !update.info.available">
                  You're on the latest version
                </template>
                <template v-else-if="update.error">
                  {{ update.error }}
                </template>
                <template v-else>
                  Click "Check Now" to check for updates
                </template>
              </div>
              <div v-if="update.lastChecked" class="text-[10px] text-muted-foreground mt-0.5">
                Last checked: {{ update.lastChecked.toLocaleTimeString() }}
              </div>
            </div>

            <!-- Check Now button -->
            <Button
              size="sm"
              variant="outline"
              :disabled="update.checking || update.installing"
              @click="update.check()"
            >
              <RefreshCw class="size-3.5 mr-1" :class="{ 'animate-spin': update.checking }" :stroke-width="1.75" />
              Check Now
            </Button>
          </div>

          <!-- Update details + install -->
          <div v-if="updateAvailable" class="border-t border-border pt-4">
            <!-- Release notes -->
            <div v-if="update.body" class="mb-4">
              <div class="text-[11px] font-medium text-muted-foreground uppercase tracking-wide mb-1.5">
                Release Notes
              </div>
              <div class="text-[12px] text-muted-foreground whitespace-pre-wrap rounded-md bg-muted/50 p-3 max-h-[200px] overflow-y-auto">
                {{ update.body }}
              </div>
            </div>

            <!-- Download progress -->
            <div v-if="update.installing" class="mb-4">
              <div class="h-1.5 w-full rounded-full bg-muted overflow-hidden">
                <div
                  class="h-full bg-primary transition-all duration-140 ease-grok"
                  :style="{ width: update.extracting ? '100%' : `${update.progress}%` }"
                />
              </div>
              <div class="text-[11px] text-muted-foreground mt-1.5 flex items-center gap-1.5">
                <Loader2 class="size-3 animate-spin" :stroke-width="1.75" />
                {{ update.extracting ? "Extracting and installing..." : `Downloading... ${Math.round(update.progress)}%` }}
              </div>
            </div>

            <!-- Install button -->
            <div v-else class="flex flex-wrap items-center gap-2">
              <Button size="sm" @click="update.install()">
                <Download class="size-3.5 mr-1" :stroke-width="1.75" />
                Download & Restart
              </Button>
              <Button size="sm" variant="outline" @click="update.showUpdateDialog()">
                <ArrowUpCircle class="size-3.5 mr-1" :stroke-width="1.75" />
                Show Update Dialog
              </Button>
            </div>
          </div>

          <!-- Error retry -->
          <div v-if="update.error && !update.installing" class="border-t border-border pt-4">
            <Button size="sm" variant="outline" @click="update.check()">
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
