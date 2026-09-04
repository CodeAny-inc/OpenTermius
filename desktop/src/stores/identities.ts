import { defineStore } from "pinia";
import { ref, computed } from "vue";
import * as api from "../api";
import type { Identity } from "../types";

export const useIdentitiesStore = defineStore("identities", () => {
  const identities = ref<Identity[]>([]);
  const searchQuery = ref("");
  const selectedGroupId = ref<string | null>(null);

  const filteredIdentities = computed(() => {
    let result = identities.value;
    if (selectedGroupId.value) {
      result = result.filter((i) => i.group_id === selectedGroupId.value);
    }
    if (searchQuery.value.trim()) {
      const q = searchQuery.value.toLowerCase();
      result = result.filter(
        (i) =>
          i.label.toLowerCase().includes(q) ||
          i.username.toLowerCase().includes(q) ||
          i.tags.some((t) => t.toLowerCase().includes(q)),
      );
    }
    return result;
  });

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

  return {
    identities,
    searchQuery,
    selectedGroupId,
    filteredIdentities,
    load,
    addIdentity,
    updateIdentity,
    deleteIdentity,
  };
});
