<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { useVaultStore } from "../stores/vault";
import Button from "./ui/Button.vue";
import Input from "./ui/Input.vue";
import FormGroup from "./ui/FormGroup.vue";
import Label from "./ui/Label.vue";
import {
  Vault as VaultIcon,
  Lock,
  Unlock,
  ShieldCheck,
  AlertTriangle,
} from "lucide-vue-next";

const vault = useVaultStore();

const passphrase = ref("");
const confirmPassphrase = ref("");
const error = ref("");

onMounted(async () => {
  await vault.checkStatus();
});

async function setup() {
  error.value = "";
  if (passphrase.value !== confirmPassphrase.value) {
    error.value = "Passphrases do not match";
    return;
  }
  if (passphrase.value.length < 8) {
    error.value = "Passphrase must be at least 8 characters";
    return;
  }
  try {
    await vault.initialize(passphrase.value);
    passphrase.value = "";
    confirmPassphrase.value = "";
  } catch (e: any) {
    error.value = String(e);
  }
}

async function unlock() {
  error.value = "";
  try {
    await vault.unlock(passphrase.value);
    passphrase.value = "";
  } catch (e: any) {
    error.value = String(e);
  }
}

async function lock() {
  await vault.lock();
}
</script>

<template>
  <div class="flex flex-col h-full overflow-hidden">
    <!-- Header -->
    <div class="flex h-11 items-center gap-2 border-b border-border px-4">
      <h2 class="text-[14px] font-semibold">Vault</h2>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto p-6">
      <div class="max-w-[480px] mx-auto">
        <!-- Status card -->
        <div class="flex items-center gap-3 rounded-lg border border-border bg-card p-4 mb-6">
          <div
            class="flex h-10 w-10 items-center justify-center rounded-md"
            :class="vault.unlocked ? 'bg-green-500/10' : 'bg-muted'"
          >
            <ShieldCheck
              v-if="vault.unlocked"
              class="size-5 text-green-500"
              :stroke-width="1.75"
            />
            <Lock v-else class="size-5 text-muted-foreground" :stroke-width="1.75" />
          </div>
          <div class="flex-1">
            <div class="text-[14px] font-medium">
              {{ vault.unlocked ? "Vault unlocked" : "Vault locked" }}
            </div>
            <div class="text-[12px] text-muted-foreground">
              {{ vault.initialized ? "Encrypted storage is active" : "No vault configured" }}
            </div>
          </div>
          <Button v-if="vault.unlocked" variant="outline" size="sm" @click="lock">
            <Lock class="size-3.5" :stroke-width="1.75" />
            Lock
          </Button>
        </div>

        <!-- Setup mode -->
        <div v-if="vault.needsSetup" class="flex flex-col gap-4">
          <div class="flex items-start gap-2.5 rounded-md border border-border bg-muted/50 p-3">
            <AlertTriangle class="size-4 text-yellow-500 shrink-0 mt-0.5" :stroke-width="1.75" />
            <div class="text-[12px] text-muted-foreground">
              Set up a master passphrase to encrypt your SSH keys and credentials. This passphrase cannot be recovered.
            </div>
          </div>
          <FormGroup>
            <Label for="setup-pass">Master passphrase</Label>
            <Input id="setup-pass" v-model="passphrase" type="password" placeholder="At least 8 characters" />
          </FormGroup>
          <FormGroup>
            <Label for="setup-confirm">Confirm passphrase</Label>
            <Input id="setup-confirm" v-model="confirmPassphrase" type="password" placeholder="Repeat passphrase" />
          </FormGroup>
          <p v-if="error" class="text-[12px] text-destructive">{{ error }}</p>
          <Button @click="setup">
            <VaultIcon class="size-3.5" :stroke-width="1.75" />
            Create Vault
          </Button>
        </div>

        <!-- Unlock mode -->
        <div v-else-if="vault.needsUnlock" class="flex flex-col gap-4">
          <FormGroup>
            <Label for="unlock-pass">Master passphrase</Label>
            <Input
              id="unlock-pass"
              v-model="passphrase"
              type="password"
              placeholder="Enter master passphrase"
              @keydown.enter="unlock"
            />
          </FormGroup>
          <p v-if="error" class="text-[12px] text-destructive">{{ error }}</p>
          <Button @click="unlock">
            <Unlock class="size-3.5" :stroke-width="1.75" />
            Unlock
          </Button>
        </div>

        <!-- Unlocked mode -->
        <div v-else class="flex flex-col gap-3">
          <p class="text-[13px] text-muted-foreground">
            Your vault is unlocked. SSH keys and credentials are accessible.
            Lock the vault when you're done to keep your data secure.
          </p>
          <Button variant="outline" @click="lock">
            <Lock class="size-3.5" :stroke-width="1.75" />
            Lock Vault Now
          </Button>
        </div>
      </div>
    </div>
  </div>
</template>
