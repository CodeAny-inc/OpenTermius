<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { useIdentitiesStore } from "../stores/identities";
import { useKeysStore } from "../stores/keys";
import Button from "./ui/Button.vue";
import Dialog from "./ui/Dialog.vue";
import Input from "./ui/Input.vue";
import Select from "./ui/Select.vue";
import Textarea from "./ui/Textarea.vue";
import FormGroup from "./ui/FormGroup.vue";
import Label from "./ui/Label.vue";
import Badge from "./ui/Badge.vue";
import {
  UserCircle,
  Plus,
  Trash2,
  Pencil,
  KeyRound,
  FolderOpen,
  Loader2,
  Sparkles,
  Upload,
} from "lucide-vue-next";
import { open } from "@tauri-apps/plugin-dialog";
import * as api from "../api";
import type { Identity, AuthMethod, KeyMeta } from "../types";

const identities = useIdentitiesStore();
const keys = useKeysStore();

const showForm = ref(false);
const editing = ref<Identity | null>(null);

// Inline key creation state
const keyMode = ref<"select" | "generate" | "import">("select");
const newKeyLabel = ref("");
const importKeyText = ref("");
const importKeyPassphrase = ref("");
const creatingKey = ref(false);
const browsingFile = ref(false);

const form = ref({
  label: "",
  username: "",
  authMode: "agent" as "password" | "publickey" | "agent",
  credentialKey: "",
  keyId: "",
  tags: "",
});

onMounted(() => {
  identities.load();
  keys.load();
});

function addIdentity() {
  editing.value = null;
  form.value = {
    label: "",
    username: "",
    authMode: "agent",
    credentialKey: "",
    keyId: "",
    tags: "",
  };
  keyMode.value = "select";
  newKeyLabel.value = "";
  importKeyText.value = "";
  importKeyPassphrase.value = "";
  showForm.value = true;
}

function editIdentity(id: Identity) {
  editing.value = id;
  const authMode: "password" | "publickey" | "agent" =
    typeof id.auth === "string" ? id.auth : "password";
  const credentialKey =
    typeof id.auth === "object" && "password" in id.auth
      ? id.auth.password.credential_key
      : "";
  form.value = {
    label: id.label,
    username: id.username,
    authMode,
    credentialKey,
    keyId: id.key_id || "",
    tags: id.tags.join(", "),
  };
  keyMode.value = "select";
  newKeyLabel.value = "";
  importKeyText.value = "";
  importKeyPassphrase.value = "";
  showForm.value = true;
}

function buildAuth(): AuthMethod {
  if (form.value.authMode === "publickey") return "publickey";
  if (form.value.authMode === "agent") return "agent";
  return {
    password: { credential_key: form.value.credentialKey || "default" },
  };
}

const canSave = computed(() => {
  if (!form.value.label.trim() || !form.value.username.trim()) return false;
  if (form.value.authMode !== "publickey") return true;
  // For publickey: need either an existing key selected or a new key being created
  if (keyMode.value === "select") return !!form.value.keyId;
  if (keyMode.value === "generate") return !!newKeyLabel.value.trim();
  if (keyMode.value === "import") return !!newKeyLabel.value.trim() && !!importKeyText.value.trim();
  return false;
});

async function ensureKey(): Promise<string | null> {
  // If selecting an existing key, return it
  if (keyMode.value === "select") {
    return form.value.keyId || null;
  }
  // Generate a new key
  if (keyMode.value === "generate") {
    if (!newKeyLabel.value.trim()) return null;
    creatingKey.value = true;
    try {
      const key = await keys.generateKey(newKeyLabel.value.trim());
      return key.id;
    } catch (e) {
      alert(`Failed to generate key: ${e}`);
      return null;
    } finally {
      creatingKey.value = false;
    }
  }
  // Import a key
  if (keyMode.value === "import") {
    if (!newKeyLabel.value.trim() || !importKeyText.value.trim()) return null;
    creatingKey.value = true;
    try {
      const key = await keys.importKey(
        newKeyLabel.value.trim(),
        importKeyText.value,
        importKeyPassphrase.value || null,
      );
      return key.id;
    } catch (e) {
      alert(`Failed to import key: ${e}`);
      return null;
    } finally {
      creatingKey.value = false;
    }
  }
  return null;
}

async function save() {
  if (!canSave.value) return;
  let keyId: string | null = null;
  if (form.value.authMode === "publickey") {
    keyId = await ensureKey();
    if (!keyId) return; // error was shown by ensureKey
  }
  const identity: Identity = {
    id: editing.value?.id ?? "",
    label: form.value.label.trim(),
    username: form.value.username.trim(),
    auth: buildAuth(),
    key_id: keyId,
    tags: form.value.tags
      .split(",")
      .map((t) => t.trim())
      .filter(Boolean),
  };
  try {
    if (editing.value) {
      await identities.updateIdentity(identity);
    } else {
      await identities.addIdentity(identity);
    }
    showForm.value = false;
  } catch (e) {
    alert(`Failed to save identity: ${e}`);
  }
}

async function deleteIdentity(id: Identity) {
  if (confirm(`Delete identity "${id.label}"?`)) {
    await identities.deleteIdentity(id.id);
  }
}

async function browseForKeyFile() {
  browsingFile.value = true;
  try {
    const selected = await open({
      multiple: false,
      directory: false,
      filters: [
        { name: "SSH Private Keys", extensions: ["pem", "key", "id_rsa", "id_ed25519", "id_ecdsa", "id_dsa"] },
        { name: "All Files", extensions: ["*"] },
      ],
    });
    if (typeof selected === "string" && selected) {
      const content = await api.readKeyFile(selected);
      importKeyText.value = content;
    }
  } catch (e) {
    console.error("Failed to read key file:", e);
    alert(`Failed to read key file: ${e}`);
  } finally {
    browsingFile.value = false;
  }
}

function authLabel(auth: AuthMethod): string {
  if (auth === "publickey") return "SSH Key";
  if (auth === "agent") return "Agent";
  return "Password";
}
</script>

<template>
  <div class="flex flex-col h-full overflow-hidden">
    <!-- Header -->
    <div class="flex h-11 items-center gap-2 border-b border-border px-4">
      <h2 class="text-[14px] font-semibold">Identities</h2>
      <div class="ml-auto">
        <Button size="sm" @click="addIdentity">
          <Plus class="size-3.5" :stroke-width="1.75" />
          Add Identity
        </Button>
      </div>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto p-3">
      <div v-if="identities.identities.length" class="flex flex-col gap-1.5">
        <div
          v-for="id in identities.identities"
          :key="id.id"
          class="group flex items-center gap-3 rounded-md border border-border bg-card p-3 transition-colors duration-100 hover:border-muted-foreground/30"
        >
          <div class="flex h-9 w-9 items-center justify-center rounded-md bg-muted">
            <UserCircle class="size-4 text-muted-foreground" :stroke-width="1.75" />
          </div>
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-2">
              <span class="text-[13px] font-medium truncate">{{ id.label }}</span>
              <Badge>{{ authLabel(id.auth) }}</Badge>
            </div>
            <div class="text-[11px] text-muted-foreground truncate font-mono mt-0.5">
              {{ id.username }}
              <span v-if="id.key_id" class="ml-1.5 inline-flex items-center gap-0.5">
                <KeyRound class="size-2.5" :stroke-width="2" />
                {{ keys.keys.find(k => k.id === id.key_id)?.label ?? 'Key' }}
              </span>
            </div>
          </div>
          <div class="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
            <button
              class="flex h-7 w-7 items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-foreground transition-colors duration-100"
              aria-label="Edit identity"
              @click="editIdentity(id)"
            >
              <Pencil class="size-3.5" :stroke-width="1.75" />
            </button>
            <button
              class="flex h-7 w-7 items-center justify-center rounded text-muted-foreground hover:bg-destructive/20 hover:text-destructive transition-colors duration-100"
              aria-label="Delete identity"
              @click="deleteIdentity(id)"
            >
              <Trash2 class="size-3.5" :stroke-width="1.75" />
            </button>
          </div>
        </div>
      </div>

      <!-- Empty state -->
      <div v-else class="flex flex-col items-center justify-center py-16 px-6 gap-3 text-center">
        <UserCircle class="size-8 text-muted-foreground/50" :stroke-width="1.5" />
        <div>
          <p class="text-[14px] font-medium text-foreground">No identities yet</p>
          <p class="text-[12px] text-muted-foreground mt-1">
            Create reusable identities (username + auth + key) to apply to multiple hosts
          </p>
        </div>
        <Button size="sm" @click="addIdentity">
          <Plus class="size-3.5" :stroke-width="1.75" />
          Add Identity
        </Button>
      </div>
    </div>

    <!-- Add/Edit dialog -->
    <Dialog
      v-if="showForm"
      :open="true"
      :title="editing ? 'Edit Identity' : 'Add Identity'"
      description="An identity bundles a username and authentication method for reuse across hosts"
      width="520px"
      @close="showForm = false"
    >
      <div class="flex flex-col gap-4">
        <FormGroup>
          <Label for="id-label">Label</Label>
          <Input id="id-label" v-model="form.label" placeholder="Work admin, Personal key..." />
        </FormGroup>

        <FormGroup>
          <Label for="id-username">Username</Label>
          <Input id="id-username" v-model="form.username" placeholder="root, admin, deploy..." />
        </FormGroup>

        <FormGroup>
          <Label>Authentication method</Label>
          <Select v-model="form.authMode">
            <option value="agent">SSH Agent</option>
            <option value="publickey">SSH Key</option>
            <option value="password">Password</option>
          </Select>
        </FormGroup>

        <FormGroup v-if="form.authMode === 'password'">
          <Label for="id-cred">Credential key</Label>
          <Input id="id-cred" v-model="form.credentialKey" placeholder="Vault credential key" />
        </FormGroup>

        <!-- SSH Key section with inline create/import -->
        <template v-else-if="form.authMode === 'publickey'">
          <!-- Key mode toggle -->
          <div class="flex gap-1 p-1 rounded-md bg-muted">
            <button
              class="flex-1 h-7 rounded text-[12px] font-medium transition-colors duration-100"
              :class="keyMode === 'select' ? 'bg-background text-foreground shadow-sm' : 'text-muted-foreground'"
              @click="keyMode = 'select'"
            >
              Select existing
            </button>
            <button
              class="flex-1 h-7 rounded text-[12px] font-medium transition-colors duration-100"
              :class="keyMode === 'generate' ? 'bg-background text-foreground shadow-sm' : 'text-muted-foreground'"
              @click="keyMode = 'generate'"
            >
              <Sparkles class="inline size-3 mr-1" :stroke-width="1.75" />
              Generate new
            </button>
            <button
              class="flex-1 h-7 rounded text-[12px] font-medium transition-colors duration-100"
              :class="keyMode === 'import' ? 'bg-background text-foreground shadow-sm' : 'text-muted-foreground'"
              @click="keyMode = 'import'"
            >
              <Upload class="inline size-3 mr-1" :stroke-width="1.75" />
              Import
            </button>
          </div>

          <!-- Select existing key -->
          <FormGroup v-if="keyMode === 'select'">
            <Label for="id-key">SSH Key</Label>
            <Select id="id-key" v-model="form.keyId">
              <option value="">Select a key...</option>
              <option v-for="key in keys.keys" :key="key.id" :value="key.id">
                {{ key.label }} ({{ key.key_type }})
              </option>
            </Select>
            <p v-if="!keys.keys.length" class="text-[11px] text-muted-foreground mt-1">
              No keys available. Use Generate or Import to create one.
            </p>
          </FormGroup>

          <!-- Generate new key -->
          <template v-else-if="keyMode === 'generate'">
            <FormGroup>
              <Label for="gen-key-label">Key label</Label>
              <Input
                id="gen-key-label"
                v-model="newKeyLabel"
                placeholder="e.g. Work Laptop Key"
              />
            </FormGroup>
            <p class="text-[12px] text-muted-foreground -mt-2">
              A new Ed25519 key pair will be generated and stored encrypted in the vault.
              This key will be linked to the identity automatically.
            </p>
          </template>

          <!-- Import key -->
          <template v-else-if="keyMode === 'import'">
            <FormGroup>
              <Label for="imp-key-label">Key label</Label>
              <Input
                id="imp-key-label"
                v-model="newKeyLabel"
                placeholder="e.g. Imported Server Key"
              />
            </FormGroup>
            <FormGroup>
              <div class="flex items-center justify-between">
                <Label for="imp-key-private">Private key (OpenSSH)</Label>
                <button
                  class="inline-flex h-6 items-center gap-1 rounded text-[11px] text-muted-foreground hover:text-foreground transition-colors duration-100"
                  :disabled="browsingFile"
                  @click="browseForKeyFile"
                >
                  <Loader2 v-if="browsingFile" class="size-3 animate-spin" :stroke-width="1.75" />
                  <FolderOpen v-else class="size-3" :stroke-width="1.75" />
                  Browse...
                </button>
              </div>
              <Textarea
                id="imp-key-private"
                v-model="importKeyText"
                :rows="5"
                placeholder="-----BEGIN OPENSSH PRIVATE KEY-----... or click Browse"
                class="font-mono text-[11px]"
              />
            </FormGroup>
            <FormGroup>
              <Label for="imp-key-pass">Passphrase (optional)</Label>
              <Input
                id="imp-key-pass"
                v-model="importKeyPassphrase"
                type="password"
                placeholder="Leave empty if no passphrase"
              />
            </FormGroup>
          </template>
        </template>

        <FormGroup>
          <Label for="id-tags">Tags (comma-separated)</Label>
          <Input id="id-tags" v-model="form.tags" placeholder="work, production" />
        </FormGroup>
      </div>

      <template #footer>
        <Button variant="ghost" @click="showForm = false">Cancel</Button>
        <Button :disabled="!canSave || creatingKey" @click="save">
          <Loader2 v-if="creatingKey" class="size-3.5 mr-1 animate-spin" :stroke-width="1.75" />
          {{ editing ? "Save changes" : "Add identity" }}
        </Button>
      </template>
    </Dialog>
  </div>
</template>
