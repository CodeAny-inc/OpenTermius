<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { useHostsStore } from "../stores/hosts";
import { useKeysStore } from "../stores/keys";
import Dialog from "./ui/Dialog.vue";
import Button from "./ui/Button.vue";
import Input from "./ui/Input.vue";
import Select from "./ui/Select.vue";
import FormGroup from "./ui/FormGroup.vue";
import Label from "./ui/Label.vue";
import type { Host, AuthMethod } from "../types";

const props = defineProps<{ host: Host | null }>();
const emit = defineEmits<{ close: [] }>();

const hosts = useHostsStore();
const keys = useKeysStore();

const form = ref({
  label: "",
  hostname: "",
  port: 22,
  username: "",
  authMode: "password" as "password" | "publickey" | "agent",
  credentialKey: "",
  keyId: "",
  groupId: "",
  tags: "",
});

watch(
  () => props.host,
  (host) => {
    if (host) {
      const authMode: "password" | "publickey" | "agent" =
        typeof host.auth === "string" ? host.auth : "password";
      const credentialKey =
        typeof host.auth === "object" && "password" in host.auth
          ? host.auth.password.credential_key
          : "";
      form.value = {
        label: host.label,
        hostname: host.hostname,
        port: host.port,
        username: host.username,
        authMode,
        credentialKey,
        keyId: host.key_id || "",
        groupId: host.group_id || "",
        tags: host.tags.join(", "),
      };
    } else {
      form.value = {
        label: "",
        hostname: "",
        port: 22,
        username: "",
        authMode: "password",
        credentialKey: "",
        keyId: "",
        groupId: "",
        tags: "",
      };
    }
  },
  { immediate: true },
);

const isValid = computed(() => {
  return (
    form.value.label.trim() &&
    form.value.hostname.trim() &&
    form.value.username.trim() &&
    form.value.port > 0
  );
});

function buildAuth(): AuthMethod {
  if (form.value.authMode === "publickey") return "publickey";
  if (form.value.authMode === "agent") return "agent";
  return {
    password: { credential_key: form.value.credentialKey || "default" },
  };
}

async function save() {
  if (!isValid.value) return;
  const host: Host = {
    id: props.host?.id ?? "",
    label: form.value.label.trim(),
    hostname: form.value.hostname.trim(),
    port: form.value.port,
    username: form.value.username.trim(),
    group_id: form.value.groupId || null,
    key_id: form.value.authMode === "publickey" ? form.value.keyId || null : null,
    auth: buildAuth(),
    tags: form.value.tags
      .split(",")
      .map((t) => t.trim())
      .filter(Boolean),
    startup_command: props.host?.startup_command ?? null,
    proxy_command: props.host?.proxy_command ?? null,
    jump_host_id: props.host?.jump_host_id ?? null,
  };
  if (props.host) {
    await hosts.updateHost(host);
  } else {
    await hosts.addHost(host);
  }
  emit("close");
}
</script>

<template>
  <Dialog
    :open="true"
    :title="host ? 'Edit Host' : 'Add Host'"
    description="Configure SSH connection details"
    width="520px"
    @close="emit('close')"
  >
    <div class="flex flex-col gap-4">
      <FormGroup>
        <Label for="host-label">Label</Label>
        <Input id="host-label" v-model="form.label" placeholder="My server" />
      </FormGroup>

      <div class="grid grid-cols-3 gap-3">
        <FormGroup class="col-span-2">
          <Label for="host-hostname">Hostname</Label>
          <Input id="host-hostname" v-model="form.hostname" placeholder="example.com or 1.2.3.4" />
        </FormGroup>
        <FormGroup>
          <Label for="host-port">Port</Label>
          <Input id="host-port" v-model.number="form.port" type="number" />
        </FormGroup>
      </div>

      <FormGroup>
        <Label for="host-username">Username</Label>
        <Input id="host-username" v-model="form.username" placeholder="root" />
      </FormGroup>

      <FormGroup>
        <Label>Authentication method</Label>
        <Select v-model="form.authMode">
          <option value="password">Password</option>
          <option value="publickey">SSH Key</option>
          <option value="agent">SSH Agent</option>
        </Select>
      </FormGroup>

      <FormGroup v-if="form.authMode === 'password'">
        <Label for="host-cred">Credential key</Label>
        <Input id="host-cred" v-model="form.credentialKey" placeholder="Vault credential key (e.g. 'default')" />
      </FormGroup>

      <FormGroup v-else-if="form.authMode === 'publickey'">
        <Label for="host-key">SSH Key</Label>
        <Select id="host-key" v-model="form.keyId">
          <option value="">Select a key...</option>
          <option v-for="key in keys.keys" :key="key.id" :value="key.id">
            {{ key.label }} ({{ key.key_type }})
          </option>
        </Select>
        <p v-if="!keys.keys.length" class="text-[11px] text-muted-foreground mt-1">
          No keys available. Add one in the Keys section.
        </p>
      </FormGroup>

      <FormGroup>
        <Label for="host-group">Group (optional)</Label>
        <Select id="host-group" v-model="form.groupId">
          <option value="">No group</option>
          <option v-for="group in hosts.groups" :key="group.id" :value="group.id">
            {{ group.name }}
          </option>
        </Select>
      </FormGroup>

      <FormGroup>
        <Label for="host-tags">Tags (comma-separated)</Label>
        <Input id="host-tags" v-model="form.tags" placeholder="production, web" />
      </FormGroup>
    </div>

    <template #footer>
      <Button variant="ghost" @click="emit('close')">Cancel</Button>
      <Button :disabled="!isValid" @click="save">
        {{ host ? "Save changes" : "Add host" }}
      </Button>
    </template>
  </Dialog>
</template>
