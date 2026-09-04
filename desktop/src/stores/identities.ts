import { defineStore } from "pinia";
import { ref } from "vue";
import * as api from "../api";
import type { Identity } from "../types";

export const useIdentitiesStore = defineStore("identities", () => {
  const identities = ref<Identity[]>([]);

  async function load() {
    identities.value = await api.listIdentities();
  }

  async function addIdentity(identity: Identity) {
    const saved = await api.addIdentity(identity);
    identities.value.push(saved);
    return saved;
  }

  async function updateIdentity(identity: Identity) {
    const saved = await api.updateIdentity(identity);
    const idx = identities.value.findIndex((i) => i.id === saved.id);
    if (idx >= 0) identities.value[idx] = saved;
    return saved;
  }

  async function deleteIdentity(id: string) {
    await api.deleteIdentity(id);
    identities.value = identities.value.filter((i) => i.id !== id);
  }

  return { identities, load, addIdentity, updateIdentity, deleteIdentity };
});
