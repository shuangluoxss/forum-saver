<template>
  <n-form :model="model" :label-placement="effectiveLabelPlacement" label-width="auto">
    <!-- If only one group and it's nameless, just show the fields without collapse -->
    <template v-if="isSingleDefaultGroup && groups">
      <n-grid :cols="24" :x-gap="12">
        <template v-for="field in getVisibleFields(groups[0].fields)" :key="field.key">
          <n-form-item-gi :label="field.label" :span="field.span || 24">
            <render-field :field="field" />
          </n-form-item-gi>
        </template>
      </n-grid>
    </template>

    <div v-else class="groups-container">
      <template v-for="(group, index) in groups" :key="index">
        <template v-if="hasVisibleFields(group.fields)">
          <!-- Wrap each group in its own n-collapse to isolate animations and prevent layout jumping -->
          <n-collapse :default-expanded-names="expandedGroups" :arrow-placement="'right'" class="isolated-collapse">
            <n-collapse-item :name="group.name || 'default'" :disabled="!isCollapsible(group)"
              :class="{ 'non-collapsible-item': !isCollapsible(group) }">
              <template #header>
                <n-text strong style="font-size: 1.05rem">{{
                  group.name
                  }}</n-text>
              </template>

              <!-- Hide arrow for non-collapsible groups via CSS -->
              <template v-if="!isCollapsible(group)" #header-extra>
                <span class="no-arrow-spacer"></span>
              </template>

              <n-grid :cols="24" :x-gap="12">
                <template v-for="field in getVisibleFields(group.fields)" :key="field.key">
                  <n-form-item-gi :label="field.label" :span="field.span || 24">
                    <render-field :field="field" />
                  </n-form-item-gi>
                </template>
              </n-grid>
            </n-collapse-item>
          </n-collapse>
        </template>
      </template>
    </div>
  </n-form>
</template>

<script setup lang="ts">
import {
  NInput, NInputNumber, NSwitch, NSelect, NDynamicTags, NInputGroup, NButton, NCollapse, NCollapseItem, NText
} from "naive-ui";
import type { FormField, FormGroupConfig } from "../types";

// Helper for component dynamic rendering
const getComponent = (type: string) => {
  switch (type) {
    case "input":
    case "password":
    case "textarea":
      return NInput;
    case "number":
      return NInputNumber;
    case "switch":
      return NSwitch;
    case "select":
      return NSelect;
    case "tags":
      return NDynamicTags;
    case "auth-method":
      return resolveComponent("AuthMethodEditor");
    default:
      return NInput;
  }
};

const props = defineProps<{
  model: any;
  groups?: FormGroupConfig[];
  advancedMode?: boolean;
  labelPlacement?: "top" | "left";
}>();

const emit = defineEmits(["update:model", "pick-dir"]);

// Internal component to render fields consistently
const RenderField = (propsInner: { field: FormField }) => {
  const { field } = propsInner;
  const value = getValue(field.key);

  if (field.key === "store_dir") {
    return h(NInputGroup, null, {
      default: () => [
        h(NInput, {
          value: value,
          placeholder: field.placeholder,
          onUpdateValue: (val: string) => setValue(field.key, val),
        }),
        h(
          NButton,
          {
            type: "primary",
            onClick: () => emit("pick-dir"),
          },
          { default: () => "选择" },
        ),
      ],
    });
  }

  const component = getComponent(field.type);
  const componentProps = getComponentProps(field);

  return h(component as any, {
    ...componentProps,
    value: value,
    "model-value": value, // Keep for custom components like AuthMethodEditor
    onUpdateValue: (val: any) => setValue(field.key, val),
    "onUpdate:model-value": (val: any) => setValue(field.key, val),
  });
};

// Responsive label placement
const windowWidth = ref(window.innerWidth);
const updateWidth = () => {
  windowWidth.value = window.innerWidth;
};
onMounted(() => window.addEventListener("resize", updateWidth));
onUnmounted(() => window.removeEventListener("resize", updateWidth));

const effectiveLabelPlacement = computed(() => {
  if (props.labelPlacement) return props.labelPlacement;
  // return windowWidth.value > 800 ? "left" : "top";
  return "top";
});

const isSingleDefaultGroup = computed(() => {
  return props.groups?.length === 1 && !props.groups[0].name;
});

const expandedGroups = computed(() => {
  if (!props.groups) return [];
  return props.groups
    .filter((g) => !g.collapsed)
    .map((g) => g.name || "default");
});

const isCollapsible = (group: FormGroupConfig) => {
  return group.collapsible !== false;
};

const getVisibleFields = (fields: FormField[]) => {
  if (!fields) return [];
  if (props.advancedMode) return fields;
  return fields.filter((f) => !f.advanced);
};

const hasVisibleFields = (fields: FormField[]) => {
  return getVisibleFields(fields).length > 0;
};

const getComponentProps = (field: FormField) => {
  const base: any = { placeholder: field.placeholder };
  if (field.type === "number") {
    base.min = field.min;
    base.max = field.max;
    base.style = { width: "100%" };
    base.showButton = true;
  }
  if (field.type === "select") {
    base.options = field.options;
  }
  if (field.type === "password") {
    base.type = "password";
    base.showPasswordOn = "click";
  }
  if (field.type === "textarea") {
    base.type = "textarea";
  }
  return base;
};

const initializeDefaults = () => {
  if (!props.groups) return;
  const newModel = { ...props.model };
  let changed = false;

  props.groups.forEach((group) => {
    if (!group.fields) return;
    group.fields.forEach((field) => {
      if (field.defaultValue !== undefined) {
        const currentVal = getValueFrom(newModel, field.key);
        if (currentVal === undefined) {
          setValueIn(newModel, field.key, field.defaultValue);
          changed = true;
        }
      }
    });
  });

  if (changed) {
    emit("update:model", newModel);
  }
};

onMounted(() => {
  initializeDefaults();
});

watch(
  () => props.groups,
  () => {
    initializeDefaults();
  },
  { deep: true },
);

const getValueFrom = (obj: any, key: string) => {
  if (!obj) return undefined;
  const parts = key.split(".");
  let current = obj;
  for (const part of parts) {
    if (current === undefined || current === null) return undefined;
    current = current[part];
  }
  return current;
};

const getValue = (key: string) => {
  const val = getValueFrom(props.model, key);
  // Ensure n-input-number gets null instead of undefined if not set
  return val === undefined ? null : val;
};

const setValueIn = (obj: any, key: string, value: any) => {
  const parts = key.split(".");
  let current = obj;
  for (let i = 0; i < parts.length - 1; i++) {
    const part = parts[i];
    if (current[part] === undefined || current[part] === null) {
      current[part] = {};
    }
    // Clone to maintain reactivity when emitting the whole object
    if (Array.isArray(current[part])) {
      current[part] = [...current[part]];
    } else {
      current[part] = { ...current[part] };
    }
    current = current[part];
  }
  current[parts[parts.length - 1]] = value;
};

const setValue = (key: string, value: any) => {
  const newModel = { ...props.model };
  setValueIn(newModel, key, value);
  emit("update:model", newModel);
};
</script>

<style scoped>
.groups-container :deep(.non-collapsible-item .n-collapse-item__header) {
  cursor: default !important;
}

.groups-container :deep(.non-collapsible-item .n-collapse-item__header-main) {
  padding-left: 0 !important;
}

/* Hide arrow for disabled items */
.groups-container :deep(.non-collapsible-item .n-collapse-item-arrow) {
  display: none !important;
}

.groups-container :deep(.n-collapse-item) {
  margin-bottom: 0;
}

.isolated-collapse {
  margin-bottom: 8px;
}
</style>
