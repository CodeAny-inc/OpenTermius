<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { useHostsStore } from "../stores/hosts";
import { useKeysStore } from "../stores/keys";
import type { Host, AuthMethod } from "../types";

const props = defineProps<{ host: Host | null }>();
const emit = defineEmits<{ close: [] }>();

const hosts = useHostsStore();
const keys = useKeysStore();

const label = ref("");
const hostname = ref("");
const port = ref(22);
const username = ref("");
const groupId = ref<string | null>(null);
const authType = ref<"agent" | "password" | "publickey">("agent");
const keyId = ref<string | null>(null);
const password = ref("");
const startupCommand = ref("");
const tags = ref("");

onMounted(async () => {
  await keys.load();
  if (props.host) {
    label.value = props.host.label;
    hostname.value = props.host.hostname;
    port.value = props.host.port;
    username.value = props.host.username;
    groupId.value = props.host.group_id ?? null;
    keyId.value = props.host.key_id ?? null;
    startupCommand.value = props.host.startup_command ?? "";
    tags.value = props.host.tags.join(", ");
    if (typeof props.host.auth === "object" && "password" in props.host.auth) {
      authType.value = "password";
    } else if (props.host.auth === "publickey") {
      authType.value = "publickey";
    } else {
      authType.value = "agent";
    }
  }
});

async function save() {
  const auth: AuthMethod =
    authType.value === "password"
      ? { password: { credential_key: `host-${label.value}` } }
      : authType.value === "publickey"
        ? "publickey"
        : "agent";

  const host: Host = {
    id: props.host?.id ?? crypto.randomUUID(),
    label: label.value,
    hostname: hostname.value,
    port: port.value,
    username: username.value,
    group_id: groupId.value,
    key_id: authType.value === "publickey" ? keyId.value : null,
    auth,
    tags: tags.value
      .split(",")
      .map((t) => t.trim())
      .filter(Boolean),
    startup_command: startupCommand.value || null,
    proxy_command: null,
    jump_host_id: null,
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
  <div class="modal-overlay" @click.self="emit('close')">
    <div class="modal">
      <h2>{{ props.host ? "Edit Host" : "Add Host" }}</h2>
      <div class="form-row">
        <div class="form-group">
          <label>Label</label>
          <input v-model="label" placeholder="My Server" />
        </div>
        <div class="form-group" style="max-width: 100px;">
          <label>Port</label>
          <input v-model.number="port" type="number" />
        </div>
      </div>
      <div class="form-group">
        <label>Hostname</label>
        <input v-model="hostname" placeholder="example.com" />
      </div>
      <div class="form-row">
        <div class="form-group">
          <label>Username</label>
          <input v-model="username" placeholder="root" />
        </div>
        <div class="form-group">
          <label>Group</label>
          <select v-model="groupId">
            <option :value="null">None</option>
            <option v-for="g in hosts.groups" :key="g.id" :value="g.id">{{ g.name }}</option>
          </select>
        </div>
      </div>
      <div class="form-group">
        <label>Authentication</label>
        <select v-model="authType">
          <option value="agent">SSH Agent</option>
          <option value="password">Password</option>
          <option value="publickey">Public Key</option>
        </select>
      </div>
      <div v-if="authType === 'publickey'" class="form-group">
        <label>Key</label>
        <select v-model="keyId">
          <option :value="null">Select a key...</option>
          <option v-for="key in keys.keys" :key="key.id" :value="key.id">
            {{ key.label }} ({{ key.key_type }})
          </option>
        </select>
      </div>
      <div v-if="authType === 'password'" class="form-group">
        <label>Password (stored in OS keychain)</label>
        <input v-model="password" type="password" placeholder="Enter password" />
      </div>
      <div class="form-group">
        <label>Startup command (optional)</label>
        <input v-model="startupCommand" placeholder="e.g. docker ps" />
      </div>
      <div class="form-group">
        <label>Tags (comma-separated)</label>
        <input v-model="tags" placeholder="production, web" />
      </div>
      <div class="modal-actions">
        <button class="btn secondary" @click="emit('close')">Cancel</button>
        <button class="btn" @click="save">Save</button>
      </div>
    </div>
  </div>
</template>
