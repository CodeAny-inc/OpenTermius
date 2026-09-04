<script setup lang="ts">
import { ref, watch, nextTick } from "vue";
import { useVaultStore } from "../stores/vault";
import { useUiStore } from "../stores/ui";
import Button from "./ui/Button.vue";
import Input from "./ui/Input.vue";
import FormGroup from "./ui/FormGroup.vue";
import Label from "./ui/Label.vue";
import { Vault, Unlock, AlertCircle, Loader2 } from "lucide-vue-next";

const vault = useVaultStore();
const ui = useUiStore();

const passphrase = ref("");
const error = ref("");
const loading = ref(false);
const inputRef = ref<HTMLInputElement | null>(null);

watch(
  () => ui.showVaultUnlockModal,
  (open) => {
    if (open) {
      passphrase.value = "";
      error.value = "";
      nextTick(() => inputRef.value?.focus());
    }
  },
);

async function submit() {
  if (!passphrase.value) return;
  error.value = "";
  loading.value = true;
  try {
    await vault.unlock(passphrase.value);
    passphrase.value = "";
    ui.resolveVaultUnlock(true);
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

function cancel() {
  passphrase.value = "";
  error.value = "";
  ui.resolveVaultUnlock(false);
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="ui.showVaultUnlockModal"
      class="fixed inset-0 z-[100] flex items-center justify-center bg-black/50"
      @click.self="cancel"
    >
      <div class="w-[380px] rounded-lg border border-border bg-card p-5 shadow-xl">
        <div class="flex items-center gap-3 mb-4">
          <div class="flex h-10 w-10 items-center justify-center rounded-md bg-primary/10 shrink-0">
            <Vault class="size-5 text-primary" :stroke-width="1.75" />
          </div>
          <div>
            <h3 class="text-[14px] font-semibold">Unlock Vault</h3>
            <p class="text-[12px] text-muted-foreground mt-0.5">
              The vault must be unlocked to use SSH key authentication.
            </p>
          </div>
        </div>

        <FormGroup>
          <Label for="modal-pass">Master passphrase</Label>
          <Input
            id="modal-pass"
            ref="inputRef"
            v-model="passphrase"
            type="password"
            placeholder="Enter master passphrase"
            @keydown.enter="submit"
            @keydown.escape="cancel"
          />
        </FormGroup>

        <p v-if="error" class="text-[12px] text-destructive flex items-center gap-1.5 mt-2">
          <AlertCircle class="size-3.5" :stroke-width="1.75" />
          {{ error }}
        </p>

        <div class="flex justify-end gap-2 mt-4">
          <Button variant="ghost" size="sm" @click="cancel">Cancel</Button>
          <Button size="sm" :disabled="loading || !passphrase" @click="submit">
            <Loader2 v-if="loading" class="size-3.5 mr-1 animate-spin" :stroke-width="1.75" />
            <Unlock v-else class="size-3.5 mr-1" :stroke-width="1.75" />
            Unlock
          </Button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
