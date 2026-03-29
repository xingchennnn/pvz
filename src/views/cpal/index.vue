<template>
  <div class="cpal-page">
    <CpalHeroCard
      :status="status"
      :busy="busy"
      :loading-devices="loadingDevices"
      :can-toggle-listener="status.running && !!routing.monitorOutputDeviceId"
      @toggle-open-cpal="toggleOpenCpal"
      @toggle-listener="toggleListener(!status.listening)"
      @refresh-devices="refreshDevices"
      @refresh-status="refreshStatus"
    />

    <CpalVBCableCard
      :vbcable="vbcable"
      :vbcable-loading="vbcableLoading"
      :vbcable-setup-file-name="vbcableSetupFileName"
      @refresh-vbcable-status="refreshVBCableStatus"
      @apply-vbcable-route="applyVBCableRecommendedRoute"
      @open-vbcable-package-dir="openVBCablePackageDir"
      @open-vbcable-control-panel="openVBCableControlPanel"
      @open-vbcable-download-page="openVBCableDownloadPage"
    />

    <CpalRoutingSection
      :input-device-id="routing.inputDeviceId"
      :virtual-output-device-id="routing.virtualOutputDeviceId"
      :monitor-output-device-id="routing.monitorOutputDeviceId"
      :input-options="inputOptions"
      :virtual-output-options="virtualOutputOptions"
      :monitor-output-options="monitorOptions"
      @change-input-device-id="routing.inputDeviceId = $event"
      @change-virtual-output-device-id="routing.virtualOutputDeviceId = $event"
      @change-monitor-output-device-id="routing.monitorOutputDeviceId = $event"
    />

    <CpalStatusMetricsSection :status="status" />

    <CpalCompatibilitySection
      :model-sample-rate="status.model_sample_rate"
      :input-items="inputCompatibilityItems"
      :output-items="outputCompatibilityItems"
    />
  </div>
</template>

<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { openPath, openUrl } from "@tauri-apps/plugin-opener";
import { message } from "ant-design-vue";
import { onMounted, onUnmounted, reactive, ref, watch, computed } from "vue";
import { StorageKeys, StorageUtils } from "@/utils/storageUtils";
import CpalCompatibilitySection from "./components/CpalCompatibilitySection.vue";
import CpalHeroCard from "./components/CpalHeroCard.vue";
import CpalRoutingSection from "./components/CpalRoutingSection.vue";
import CpalStatusMetricsSection from "./components/CpalStatusMetricsSection.vue";
import CpalVBCableCard from "./components/CpalVBCableCard.vue";
import type {
  AudioDeviceCatalog,
  AudioDeviceInfo,
  AudioDevicePanelItem,
  AudioRouteSelection,
  AudioSelectOption,
  AudioStatus,
  SavedAudioRouteSelection,
  VBCableStatus,
} from "./types";

const loadSavedRouting = (): AudioRouteSelection => {
  const saved = (StorageUtils.getItem(StorageKeys.AUDIO_ROUTE_SELECTION) || {}) as SavedAudioRouteSelection;
  return {
    inputDeviceId:
      typeof saved.inputDeviceId === "string"
        ? saved.inputDeviceId
        : typeof saved.inputDeviceName === "string"
          ? saved.inputDeviceName
          : "",
    virtualOutputDeviceId:
      typeof saved.virtualOutputDeviceId === "string"
        ? saved.virtualOutputDeviceId
        : typeof saved.virtualOutputDeviceName === "string"
          ? saved.virtualOutputDeviceName
          : "",
    monitorOutputDeviceId:
      typeof saved.monitorOutputDeviceId === "string"
        ? saved.monitorOutputDeviceId
        : typeof saved.monitorOutputDeviceName === "string"
          ? saved.monitorOutputDeviceName
          : "",
  };
};

const busy = ref(false);
const loadingDevices = ref(false);
const vbcableLoading = ref(false);
const status = reactive<AudioStatus>({
  running: false,
  listening: false,
  sample_rate: 0,
  model_sample_rate: 48000,
  input_device_name: null,
  virtual_output_device_name: null,
  monitor_output_device_name: null,
  input_channels: null,
  virtual_output_channels: null,
  monitor_output_channels: null,
  input_sample_format: null,
  virtual_output_sample_format: null,
  monitor_output_sample_format: null,
  stats: {
    input_overflows: 0,
    output_underflows: 0,
    inference_frames: 0,
    inference_errors: 0,
    average_inference_ms: 0,
    last_inference_ms: 0,
  },
});
const catalog = reactive<AudioDeviceCatalog>({
  model_sample_rate: 48000,
  default_input_id: null,
  default_output_id: null,
  preferred_virtual_output_id: null,
  preferred_monitor_output_id: null,
  inputs: [],
  outputs: [],
});
const vbcable = reactive<VBCableStatus>({
  installed: false,
  package_dir: null,
  setup_path: null,
  control_panel_path: null,
  input_device_id: null,
  input_device_name: null,
  output_device_id: null,
  output_device_name: null,
  official_download_url: "https://download.vb-audio.com/Download_CABLE/VBCABLE_Driver_Pack45.zip",
  integration_notice: "",
});
const routing = reactive<AudioRouteSelection>(loadSavedRouting());

let pollTimer: number | undefined;

const compatibleInputs = computed(() => catalog.inputs.filter((device) => device.compatible));
const compatibleOutputs = computed(() => catalog.outputs.filter((device) => device.compatible));
const monitorOutputOptions = computed(() =>
  compatibleOutputs.value.filter((device) => device.id !== routing.virtualOutputDeviceId),
);

const buildDeviceLabelMap = (devices: AudioDeviceInfo[]) => {
  const baseLabels = devices.map((device) => {
    const detail = typeof device.detail === "string" ? device.detail.trim() : "";
    return detail && detail !== device.name ? `${device.name} · ${detail}` : device.name;
  });
  const totalByLabel = new Map<string, number>();
  for (const label of baseLabels) {
    totalByLabel.set(label, (totalByLabel.get(label) || 0) + 1);
  }

  const seenByLabel = new Map<string, number>();
  return new Map(
    devices.map((device, index) => {
      const baseLabel = baseLabels[index];
      const seen = (seenByLabel.get(baseLabel) || 0) + 1;
      seenByLabel.set(baseLabel, seen);
      const finalLabel =
        (totalByLabel.get(baseLabel) || 0) > 1 ? `${baseLabel} (${seen})` : baseLabel;
      return [device.id, finalLabel];
    }),
  );
};

const compatibleInputLabelMap = computed(() => buildDeviceLabelMap(compatibleInputs.value));
const compatibleOutputLabelMap = computed(() => buildDeviceLabelMap(compatibleOutputs.value));
const monitorOutputLabelMap = computed(() => buildDeviceLabelMap(monitorOutputOptions.value));
const catalogInputLabelMap = computed(() => buildDeviceLabelMap(catalog.inputs));
const catalogOutputLabelMap = computed(() => buildDeviceLabelMap(catalog.outputs));

const getDeviceLabel = (device: AudioDeviceInfo, labelMap: Map<string, string>) =>
  labelMap.get(device.id) || device.name;

const inputDeviceOptionLabel = (device: AudioDeviceInfo) =>
  getDeviceLabel(device, compatibleInputLabelMap.value);

const outputDeviceOptionLabel = (device: AudioDeviceInfo) =>
  getDeviceLabel(device, compatibleOutputLabelMap.value);

const monitorDeviceOptionLabel = (device: AudioDeviceInfo) =>
  getDeviceLabel(device, monitorOutputLabelMap.value);

const inputCatalogDeviceLabel = (device: AudioDeviceInfo) =>
  getDeviceLabel(device, catalogInputLabelMap.value);

const outputCatalogDeviceLabel = (device: AudioDeviceInfo) =>
  getDeviceLabel(device, catalogOutputLabelMap.value);

const inputOptions = computed<AudioSelectOption[]>(() =>
  compatibleInputs.value.map((device) => ({
    id: device.id,
    label: inputDeviceOptionLabel(device),
  })),
);

const virtualOutputOptions = computed<AudioSelectOption[]>(() =>
  compatibleOutputs.value.map((device) => ({
    id: device.id,
    label: outputDeviceOptionLabel(device),
    recommended: device.virtual_mic_candidate,
  })),
);

const monitorOptions = computed<AudioSelectOption[]>(() =>
  monitorOutputOptions.value.map((device) => ({
    id: device.id,
    label: monitorDeviceOptionLabel(device),
  })),
);

const inputCompatibilityItems = computed<AudioDevicePanelItem[]>(() =>
  catalog.inputs.map((device) => ({
    id: device.id,
    label: inputCatalogDeviceLabel(device),
    badgeLabel: device.compatible ? "支持直连" : "不支持直连",
    badgeVariant: device.compatible ? "success" : "muted",
    lines: [`采样格式: ${device.supported_sample_formats.join(" / ") || "无"}`],
  })),
);

const outputCompatibilityItems = computed<AudioDevicePanelItem[]>(() =>
  catalog.outputs.map((device) => ({
    id: device.id,
    label: outputCatalogDeviceLabel(device),
    badgeLabel: device.virtual_mic_candidate ? "推荐虚拟麦" : "推荐监听",
    badgeVariant: device.virtual_mic_candidate ? "primary" : "muted",
    lines: [
      `状态: ${device.compatible ? `支持 ${status.model_sample_rate}Hz 直连` : `不支持 ${status.model_sample_rate}Hz 直连`}`,
      `采样格式: ${device.supported_sample_formats.join(" / ") || "无"}`,
    ],
  })),
);

const vbcableSetupFileName = computed(() => {
  const fullPath = vbcable.setup_path;
  if (!fullPath) {
    return "VBCABLE_Setup_x64.exe";
  }

  return fullPath.split(/[/\\]/).pop() || "VBCABLE_Setup_x64.exe";
});

const copyStatus = (next: AudioStatus) => {
  Object.assign(status, next, {
    stats: { ...next.stats },
  });
};

const copyVBCableStatus = (next: VBCableStatus) => {
  Object.assign(vbcable, next);
};

const copyCatalog = (next: AudioDeviceCatalog) => {
  Object.assign(catalog, next, {
    inputs: [...next.inputs],
    outputs: [...next.outputs],
  });
};

const formatError = (error: unknown) => {
  if (typeof error === "string") {
    return error;
  }
  if (error && typeof error === "object" && "toString" in error) {
    return String(error);
  }
  return "发生未知错误";
};

const saveRouting = () => {
  StorageUtils.setItem(StorageKeys.AUDIO_ROUTE_SELECTION, { ...routing });
};

const resolveDeviceId = (devices: AudioDeviceInfo[], value: string | null | undefined) => {
  if (!value) {
    return "";
  }

  const exactMatch = devices.find((device) => device.id === value);
  if (exactMatch) {
    return exactMatch.id;
  }

  const sameNameDevices = devices.filter((device) => device.name === value);
  if (sameNameDevices.length === 1) {
    return sameNameDevices[0].id;
  }

  return "";
};

const firstCompatibleInputId = () => compatibleInputs.value[0]?.id || "";

const firstCompatibleOutputId = () => compatibleOutputs.value[0]?.id || "";

const chooseInputFallback = (excludedId?: string | null) =>
  compatibleInputs.value.find((device) => device.id !== excludedId)?.id || firstCompatibleInputId();

const chooseMonitorFallback = (excludedId: string) => {
  const preferred = compatibleOutputs.value.find(
    (device) => !device.virtual_mic_candidate && device.id !== excludedId,
  );
  if (preferred) {
    return preferred.id;
  }

  const anyCompatible = compatibleOutputs.value.find((device) => device.id !== excludedId);
  return anyCompatible?.id || "";
};

const reconcileRoutingSelection = () => {
  const nextInput =
    resolveDeviceId(compatibleInputs.value, routing.inputDeviceId) ||
    resolveDeviceId(compatibleInputs.value, catalog.default_input_id) ||
    firstCompatibleInputId();

  const nextVirtualOutput =
    resolveDeviceId(compatibleOutputs.value, routing.virtualOutputDeviceId) ||
    resolveDeviceId(compatibleOutputs.value, catalog.preferred_virtual_output_id) ||
    resolveDeviceId(compatibleOutputs.value, catalog.default_output_id) ||
    firstCompatibleOutputId();

  const preferredMonitor =
    resolveDeviceId(compatibleOutputs.value, catalog.preferred_monitor_output_id) ||
    resolveDeviceId(compatibleOutputs.value, catalog.default_output_id) ||
    chooseMonitorFallback(nextVirtualOutput);

  const currentMonitor = resolveDeviceId(
    compatibleOutputs.value,
    routing.monitorOutputDeviceId,
  );

  const nextMonitorOutput =
    currentMonitor && currentMonitor !== nextVirtualOutput
      ? currentMonitor
      : preferredMonitor !== nextVirtualOutput
        ? preferredMonitor
        : chooseMonitorFallback(nextVirtualOutput);

  routing.inputDeviceId = nextInput;
  routing.virtualOutputDeviceId = nextVirtualOutput;
  routing.monitorOutputDeviceId = nextMonitorOutput;
};

const refreshStatus = async () => {
  const next = await invoke<AudioStatus>("get_audio_status");
  copyStatus(next);
};

const refreshVBCableStatus = async () => {
  vbcableLoading.value = true;
  try {
    const next = await invoke<VBCableStatus>("get_vbcable_status");
    copyVBCableStatus(next);
  } catch (error) {
    message.error(formatError(error), 2);
  } finally {
    vbcableLoading.value = false;
  }
};

const refreshDevices = async () => {
  loadingDevices.value = true;
  try {
    const [catalogNext, vbcableNext] = await Promise.all([
      invoke<AudioDeviceCatalog>("list_audio_devices"),
      invoke<VBCableStatus>("get_vbcable_status"),
    ]);
    copyCatalog(catalogNext);
    copyVBCableStatus(vbcableNext);
    reconcileRoutingSelection();
  } catch (error) {
    message.error(formatError(error), 2);
  } finally {
    loadingDevices.value = false;
  }
};

const applyVBCableRecommendedRoute = () => {
  if (!vbcable.installed || !vbcable.output_device_id) {
    message.warning("尚未检测到可用的 VB-CABLE 设备", 2);
    return;
  }

  const outputDeviceId = resolveDeviceId(compatibleOutputs.value, vbcable.output_device_id);
  if (!outputDeviceId) {
    message.warning("已检测到 VB-CABLE，但当前设备列表中还没有可用的播放端，请先刷新设备", 2);
    return;
  }

  const inputDeviceId = resolveDeviceId(compatibleInputs.value, vbcable.input_device_id);
  routing.virtualOutputDeviceId = outputDeviceId;

  if (!routing.inputDeviceId || (inputDeviceId && routing.inputDeviceId === inputDeviceId)) {
    routing.inputDeviceId = chooseInputFallback(inputDeviceId);
  }

  if (!routing.monitorOutputDeviceId || routing.monitorOutputDeviceId === outputDeviceId) {
    routing.monitorOutputDeviceId = chooseMonitorFallback(outputDeviceId);
  }

  saveRouting();
  message.success(
    `已切换到 VB-CABLE 推荐配置，通话软件麦克风请选择 ${vbcable.input_device_name || "麦克风 (CABLE Output)"}`,
    3,
  );
};

const openVBCablePackageDir = async () => {
  if (!vbcable.package_dir) {
    message.warning("未发现本地 VB-CABLE 驱动包", 2);
    return;
  }

  await openPath(vbcable.package_dir);
  message.info(`请以管理员身份运行 ${vbcableSetupFileName.value} 完成安装`, 4);
};

const openVBCableControlPanel = async () => {
  if (!vbcable.control_panel_path) {
    message.warning("未找到 VB-CABLE 控制面板", 2);
    return;
  }

  await openPath(vbcable.control_panel_path);
};

const openVBCableDownloadPage = async () => {
  await openUrl(vbcable.official_download_url);
};

const toggleOpenCpal = async () => {
  busy.value = true;
  try {
    if (status.running) {
      if (status.listening) {
        await invoke<string>("close_listener");
      }
      const res = await invoke<string>("stop_noise_reduction");
      message.info(res, 1.5);
    } else {
      if (!routing.virtualOutputDeviceId) {
        throw new Error("请先选择虚拟麦输出设备");
      }
      if (routing.monitorOutputDeviceId && routing.monitorOutputDeviceId === routing.virtualOutputDeviceId) {
        throw new Error("监听设备不能和虚拟麦输出设备相同");
      }
      const res = await invoke<string>("start_noise_reduction", {
        inputDeviceId: routing.inputDeviceId || null,
        virtualOutputDeviceId: routing.virtualOutputDeviceId || null,
        monitorOutputDeviceId: routing.monitorOutputDeviceId || null,
      });
      message.info(res, 2);
    }
  } catch (error) {
    message.error(formatError(error), 3);
  } finally {
    await refreshStatus();
    busy.value = false;
  }
};

const toggleListener = async (open: boolean) => {
  if (open && !status.running) {
    message.warning("请先启动降噪引擎", 2);
    return;
  }
  if (open && !routing.monitorOutputDeviceId) {
    message.warning("请先选择本地监听设备", 2);
    return;
  }

  busy.value = true;
  try {
    const res = await invoke<string>(open ? "open_listener" : "close_listener");
    message.info(res, 1.5);
  } catch (error) {
    message.error(formatError(error), 3);
  } finally {
    await refreshStatus();
    busy.value = false;
  }
};

watch(
  () => ({ ...routing }),
  () => {
    if (routing.monitorOutputDeviceId === routing.virtualOutputDeviceId) {
      routing.monitorOutputDeviceId = "";
    }
    saveRouting();
  },
  { deep: true },
);

onMounted(async () => {
  await Promise.all([refreshStatus(), refreshDevices()]);
  pollTimer = window.setInterval(() => {
    refreshStatus().catch(() => undefined);
  }, 1000);
});

onUnmounted(() => {
  if (pollTimer) {
    window.clearInterval(pollTimer);
  }
});
</script>

<style>
.cpal-page {
  --cpal-page-text: #1f2937;
  --cpal-page-muted: #667085;
  --cpal-page-soft: #8b97ab;
  --cpal-card-bg: linear-gradient(180deg, #ffffff 0%, #f9fbff 100%);
  --cpal-card-bg-strong: linear-gradient(135deg, #eef5ff 0%, #f8fbff 55%, #ffffff 100%);
  --cpal-card-bg-soft: #f5f8fc;
  --cpal-card-border: #dbe3ee;
  --cpal-card-shadow: 0 10px 30px rgba(15, 23, 42, 0.06);
  --cpal-pill-bg: #eef4fb;
  --cpal-pill-text: #31435f;
  --cpal-success-bg: #e9fbf1;
  --cpal-success-text: #127a45;
  --cpal-warning-bg: #fff4df;
  --cpal-warning-text: #b26a00;
  --cpal-primary-bg: #e8f1ff;
  --cpal-primary-text: #1f5fd1;
  --cpal-muted-bg: #eef2f7;
  --cpal-muted-text: #5f6f86;
  --cpal-input-bg: #ffffff;
  --cpal-input-border: #d4dcea;
  display: grid;
  gap: 16px;
  color: var(--cpal-page-text);
}

body.dark-theme .cpal-page {
  --cpal-page-text: #eef4ff;
  --cpal-page-muted: rgba(210, 221, 241, 0.72);
  --cpal-page-soft: rgba(170, 187, 216, 0.68);
  --cpal-card-bg: linear-gradient(180deg, rgba(17, 23, 34, 0.96) 0%, rgba(13, 19, 29, 0.92) 100%);
  --cpal-card-bg-strong: linear-gradient(135deg, rgba(16, 30, 49, 0.98) 0%, rgba(15, 22, 33, 0.96) 52%, rgba(12, 17, 26, 0.94) 100%);
  --cpal-card-bg-soft: rgba(255, 255, 255, 0.04);
  --cpal-card-border: rgba(128, 149, 184, 0.18);
  --cpal-card-shadow: 0 18px 36px rgba(0, 0, 0, 0.24);
  --cpal-pill-bg: rgba(255, 255, 255, 0.06);
  --cpal-pill-text: rgba(226, 235, 252, 0.9);
  --cpal-success-bg: rgba(34, 197, 94, 0.14);
  --cpal-success-text: #7be7a7;
  --cpal-warning-bg: rgba(245, 158, 11, 0.16);
  --cpal-warning-text: #ffd079;
  --cpal-primary-bg: rgba(59, 130, 246, 0.16);
  --cpal-primary-text: #8cc0ff;
  --cpal-muted-bg: rgba(255, 255, 255, 0.06);
  --cpal-muted-text: rgba(196, 209, 233, 0.78);
  --cpal-input-bg: rgba(255, 255, 255, 0.03);
  --cpal-input-border: rgba(128, 149, 184, 0.22);
}

.cpal-card {
  border: 1px solid var(--cpal-card-border);
  border-radius: 18px;
  background: var(--cpal-card-bg);
  box-shadow: var(--cpal-card-shadow);
  padding: 18px;
}

.cpal-card--accent {
  background: var(--cpal-card-bg-strong);
}

.cpal-hero {
  display: flex;
  justify-content: space-between;
  gap: 18px;
  align-items: flex-start;
}

.cpal-hero__content {
  min-width: 0;
  flex: 1;
}

.cpal-kicker {
  color: var(--cpal-primary-text);
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.12em;
  text-transform: uppercase;
}

.cpal-page__title {
  margin-top: 6px;
  font-size: 28px;
  font-weight: 700;
  line-height: 1.15;
}

.cpal-page__desc,
.cpal-card__subtitle,
.device-item__meta,
.cpal-empty,
.cpal-info-label,
.cpal-fact-item span,
.cpal-metric-item span {
  color: var(--cpal-page-muted);
}

.cpal-page__desc {
  margin-top: 8px;
  max-width: 760px;
  font-size: 14px;
  line-height: 1.7;
}

.cpal-pills {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  margin-top: 14px;
}

.cpal-pill,
.cpal-badge,
.cpal-soft-tag {
  display: inline-flex;
  align-items: center;
  border-radius: 999px;
  padding: 6px 10px;
  font-size: 12px;
  font-weight: 600;
  line-height: 1;
}

.cpal-pill {
  background: var(--cpal-pill-bg);
  color: var(--cpal-pill-text);
}

.cpal-pill.is-success,
.cpal-badge.is-success {
  background: var(--cpal-success-bg);
  color: var(--cpal-success-text);
}

.cpal-pill.is-primary,
.cpal-badge.is-primary {
  background: var(--cpal-primary-bg);
  color: var(--cpal-primary-text);
}

.cpal-badge.is-warning {
  background: var(--cpal-warning-bg);
  color: var(--cpal-warning-text);
}

.cpal-pill.is-idle,
.cpal-badge.is-muted,
.cpal-soft-tag {
  background: var(--cpal-muted-bg);
  color: var(--cpal-muted-text);
}

.cpal-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  justify-content: flex-end;
}

.cpal-actions--compact {
  align-self: flex-start;
}

.cpal-card__head,
.cpal-card__title-row {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  align-items: center;
}

.cpal-card__head {
  align-items: flex-start;
}

.cpal-card__title {
  font-size: 18px;
  font-weight: 700;
  line-height: 1.2;
}

.cpal-card__subtitle {
  margin-top: 6px;
  font-size: 13px;
  line-height: 1.7;
}

.cpal-info-grid,
.cpal-fact-grid,
.cpal-metric-grid {
  display: grid;
  gap: 12px;
  margin-top: 16px;
}

.cpal-info-grid {
  grid-template-columns: repeat(3, minmax(0, 1fr));
}

.cpal-info-item,
.cpal-fact-item,
.cpal-metric-item,
.device-item {
  border: 1px solid var(--cpal-card-border);
  border-radius: 14px;
  background: var(--cpal-card-bg-soft);
}

.cpal-info-item,
.cpal-fact-item,
.cpal-metric-item {
  padding: 12px 14px;
}

.cpal-info-item strong,
.cpal-fact-item strong,
.cpal-metric-item strong,
.device-item__title {
  display: block;
  margin-top: 6px;
  color: var(--cpal-page-text);
  font-size: 14px;
  font-weight: 600;
  line-height: 1.55;
  word-break: break-word;
}

.cpal-note-list {
  display: grid;
  gap: 8px;
  margin-top: 14px;
}

.cpal-note {
  border-left: 3px solid var(--cpal-card-border);
  padding-left: 12px;
  color: var(--cpal-page-muted);
  font-size: 13px;
  line-height: 1.7;
}

.cpal-note--accent {
  border-left-color: var(--cpal-primary-text);
  color: var(--cpal-page-text);
}

.cpal-grid {
  display: grid;
  gap: 16px;
}

.cpal-grid--three {
  grid-template-columns: repeat(3, minmax(0, 1fr));
}

.cpal-grid--two {
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.setting-card {
  display: grid;
  gap: 12px;
}

.device-list {
  display: grid;
  gap: 10px;
  margin-top: 14px;
}

.device-item {
  padding: 12px 14px;
}

.device-item__head {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  align-items: flex-start;
}

.device-item__meta {
  margin-top: 6px;
  font-size: 13px;
  line-height: 1.6;
}

.cpal-empty {
  margin-top: 14px;
  font-size: 13px;
}

.cpal-page .ant-select-selector {
  min-height: 44px !important;
  border-radius: 12px !important;
  border-color: var(--cpal-input-border) !important;
  background: var(--cpal-input-bg) !important;
  box-shadow: none !important;
  transition: border-color 0.2s ease, background 0.2s ease !important;
}

.cpal-page .ant-select-selection-item,
.cpal-page .ant-select-selection-placeholder,
.cpal-page .ant-select-selection-search-input {
  line-height: 42px !important;
}

.cpal-page .ant-select-selection-item {
  color: var(--cpal-page-text) !important;
}

.cpal-page .ant-select-selection-placeholder {
  color: var(--cpal-page-soft) !important;
}

.cpal-page .ant-select-arrow {
  color: var(--cpal-page-soft) !important;
}

@media (max-width: 1080px) {
  .cpal-hero,
  .cpal-card__head {
    flex-direction: column;
  }

  .cpal-actions {
    justify-content: flex-start;
  }

  .cpal-grid--three,
  .cpal-grid--two,
  .cpal-info-grid {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 640px) {
  .cpal-card {
    padding: 14px;
    border-radius: 16px;
  }

  .cpal-page__title {
    font-size: 24px;
  }

  .cpal-actions {
    width: 100%;
  }

  .cpal-actions .ant-btn {
    width: 100%;
  }

  .device-item__head,
  .cpal-card__title-row {
    flex-direction: column;
    align-items: flex-start;
  }
}
</style>
