<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useWorkspacesStore } from "../stores/workspaces";
import Button from "./ui/Button.vue";
import Input from "./ui/Input.vue";
import Dialog from "./ui/Dialog.vue";
import FormGroup from "./ui/FormGroup.vue";
import Label from "./ui/Label.vue";
import {
  FolderOpen,
  Plus,
  Trash2,
  Pencil,
  Folder,
} from "lucide-vue-next";
import type { Workspace } from "../types";

const workspaces = useWorkspacesStore();

const showForm = ref(false);
const editing = ref<Workspace | null>(null);
const name = ref("");

onMounted(() => {
  workspaces.load();
});

function addWorkspace() {
  editing.value = null;
  name.value = "";
  showForm.value = true;
}

function editWorkspace(ws: Workspace) {
  editing.value = ws;
  name.value = ws.name;
  showForm.value = true;
}

async function save() {
  if (!name.value.trim()) return;
  if (editing.value) {
    await workspaces.saveWorkspace({ ...editing.value, name: name.value });
  } else {
    await workspaces.createWorkspace(name.value);
  }
  showForm.value = false;
}

async function remove(id: string, name: string) {
  if (confirm(`Delete workspace "${name}"?`)) {
    await workspaces.deleteWorkspace(id);
  }
}
</script>

<template>
  <div class="flex flex-col h-full overflow-hidden">
    <!-- Header -->
    <div class="flex h-11 items-center gap-2 border-b border-border px-4">
      <h2 class="text-[14px] font-semibold">Workspaces</h2>
      <div class="ml-auto">
        <Button size="sm" @click="addWorkspace">
          <Plus class="size-3.5" :stroke-width="1.75" />
          New Workspace
        </Button>
      </div>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto p-3">
      <div v-if="workspaces.workspaces.length" class="flex flex-col gap-1.5">
        <div
          v-for="ws in workspaces.workspaces"
          :key="ws.id"
          class="group flex items-center gap-3 rounded-md border border-border bg-card p-3 transition-colors duration-100 hover:border-muted-foreground/30"
        >
          <div class="flex h-9 w-9 items-center justify-center rounded-md bg-muted">
            <FolderOpen class="size-4 text-muted-foreground" :stroke-width="1.75" />
          </div>
          <div class="flex-1 min-w-0">
            <div class="text-[13px] font-medium truncate">{{ ws.name }}</div>
            <div class="text-[11px] text-muted-foreground truncate mt-0.5">
              {{ ws.tabs?.length || 0 }} saved tab layout{{ (ws.tabs?.length || 0) === 1 ? '' : 's' }}
            </div>
          </div>
          <div class="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
            <button
              class="flex h-7 w-7 items-center justify-center rounded text-muted-foreground hover:bg-accent hover:text-foreground transition-colors duration-100"
              aria-label="Edit workspace"
              @click="editWorkspace(ws)"
            >
              <Pencil class="size-3.5" :stroke-width="1.75" />
            </button>
            <button
              class="flex h-7 w-7 items-center justify-center rounded text-muted-foreground hover:bg-destructive/20 hover:text-destructive transition-colors duration-100"
              aria-label="Delete workspace"
              @click="remove(ws.id, ws.name)"
            >
              <Trash2 class="size-3.5" :stroke-width="1.75" />
            </button>
          </div>
        </div>
      </div>

      <!-- Empty state -->
      <div v-else class="flex flex-col items-center justify-center py-16 px-6 gap-3 text-center">
        <Folder class="size-8 text-muted-foreground/50" :stroke-width="1.5" />
        <div>
          <p class="text-[14px] font-medium text-foreground">No workspaces yet</p>
          <p class="text-[12px] text-muted-foreground mt-1">
            Save tab layouts into workspaces for different projects
          </p>
        </div>
        <Button size="sm" @click="addWorkspace">
          <Plus class="size-3.5" :stroke-width="1.75" />
          New Workspace
        </Button>
      </div>
    </div>

    <Dialog
      v-if="showForm"
      :open="true"
      :title="editing ? 'Edit Workspace' : 'New Workspace'"
      width="440px"
      @close="showForm = false"
    >
      <div class="flex flex-col gap-4">
        <FormGroup>
          <Label for="ws-name">Name</Label>
          <Input id="ws-name" v-model="name" placeholder="Production" @keydown.enter="save" />
        </FormGroup>
      </div>
      <template #footer>
        <Button variant="ghost" @click="showForm = false">Cancel</Button>
        <Button :disabled="!name.trim()" @click="save">
          {{ editing ? "Save changes" : "Create workspace" }}
        </Button>
      </template>
    </Dialog>
  </div>
</template>
