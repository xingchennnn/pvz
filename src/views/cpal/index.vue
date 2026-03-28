<template>
  <div class="c-[var(--text-color)] text-18px font-600">降噪设置</div>
  <div class="mt-12px flex flex-wrap gap-12px">
    <a-button :loading="busy" @click="toggleOpenCpal" type="primary">
      {{ status.running ? "停止降噪" : "开始降噪" }}
    </a-button>
    <a-button
      :loading="busy"
      @click="toggleListener(!status.listening)"
      :disabled="!status.running || !routing.monitorOutputDeviceName"
    >
      {{ status.listening ? "关闭监听" : "开启监听" }}
    </a-button>
    <a-button :loading="loadingDevices" @click="refreshDevices">
      刷新设备
    </a-button>
    <a-button :loading="busy" @click="refreshStatus">刷新状态</a-button>
  </div>

  <div class="mt-16px grid gap-16px md:grid-cols-3">
    <div class="rounded-10px border border-[#dbe2ea] p-14px">
      <div class="mb-8px text-15px font-600">输入麦克风</div>
      <a-select v-model:value="routing.inputDeviceName" class="w-full" placeholder="选择输入麦克风">
        <a-select-option
          v-for="device in compatibleInputs"
          :key="`route-input-${device.name}`"
          :value="device.name"
        >
          {{ device.name }}
        </a-select-option>
      </a-select>
      <div class="mt-8px text-12px leading-20px text-[#6b7280]">
        原始麦克风采集源。修改后需要重新启动降噪引擎。
      </div>
    </div>

    <div class="rounded-10px border border-[#dbe2ea] p-14px">
      <div class="mb-8px text-15px font-600">虚拟麦输出</div>
      <a-select
        v-model:value="routing.virtualOutputDeviceName"
        class="w-full"
        placeholder="选择虚拟麦输出设备"
      >
        <a-select-option
          v-for="device in compatibleOutputs"
          :key="`route-virtual-${device.name}`"
          :value="device.name"
        >
          {{ device.name }}{{ device.virtual_mic_candidate ? " · 推荐" : "" }}
        </a-select-option>
      </a-select>
      <div class="mt-8px text-12px leading-20px text-[#6b7280]">
        处理后的声音会持续送到这里。若使用 VB-CABLE，应选择播放端 CABLE Input。
      </div>
    </div>

    <div class="rounded-10px border border-[#dbe2ea] p-14px">
      <div class="mb-8px text-15px font-600">本地监听设备</div>
      <a-select
        v-model:value="routing.monitorOutputDeviceName"
        class="w-full"
        placeholder="可选，不监听可留空"
      >
        <a-select-option value="">不启用本地监听</a-select-option>
        <a-select-option
          v-for="device in monitorOutputOptions"
          :key="`route-monitor-${device.name}`"
          :value="device.name"
        >
          {{ device.name }}
        </a-select-option>
      </a-select>
      <div class="mt-8px text-12px leading-20px text-[#6b7280]">
        “开启监听”只控制这一路，不影响虚拟麦输出。
      </div>
    </div>
  </div>

  <div class="mt-16px rounded-10px border border-[#dbe2ea] bg-[#f9fafb] p-14px text-13px leading-22px text-[#374151]">
    当前链路: 麦克风输入 -> RNNoise 处理 -> 虚拟麦输出设备。开启监听后，会额外复制一份到本地监听设备。
  </div>

  <div class="mt-16px rounded-10px bg-[#f5f7fa] p-14px text-14px leading-22px text-[#1f2937]">
    <div>引擎状态: {{ status.running ? "已启动" : "未启动" }}</div>
    <div>监听状态: {{ status.listening ? "已打开" : "已关闭" }}</div>
    <div>模型目标采样率: {{ status.model_sample_rate }} Hz</div>
    <div>当前采样率: {{ status.sample_rate || "未建立链路" }}</div>
    <div>输入设备: {{ status.input_device_name || "未绑定" }}</div>
    <div>虚拟麦输出: {{ status.virtual_output_device_name || "未绑定" }}</div>
    <div>监听设备: {{ status.monitor_output_device_name || "未配置" }}</div>
    <div>
      输入格式: {{ status.input_sample_format || "-" }} / {{ status.input_channels || "-" }} 声道
    </div>
    <div>
      虚拟麦格式: {{ status.virtual_output_sample_format || "-" }} / {{ status.virtual_output_channels || "-" }} 声道
    </div>
    <div>
      监听格式: {{ status.monitor_output_sample_format || "-" }} / {{ status.monitor_output_channels || "-" }} 声道
    </div>
  </div>

  <div class="mt-16px grid gap-16px md:grid-cols-2">
    <div class="rounded-10px border border-[#dbe2ea] p-14px">
      <div class="mb-8px text-15px font-600">输入设备兼容性</div>
      <div class="text-12px text-[#6b7280]">
        当前实现只接受可直接工作在 {{ status.model_sample_rate }} Hz 的设备。
      </div>
      <div v-if="catalog.inputs.length" class="mt-10px space-y-8px">
        <div
          v-for="device in catalog.inputs"
          :key="`input-${device.name}`"
          class="rounded-8px bg-[#f8fafc] px-10px py-8px text-13px"
        >
          <div class="font-600">{{ device.name }}</div>
          <div>
            状态: {{ device.compatible ? `支持 ${status.model_sample_rate}Hz 直连` : `不支持 ${status.model_sample_rate}Hz 直连` }}
          </div>
          <div>采样格式: {{ device.supported_sample_formats.join(" / ") || "无" }}</div>
        </div>
      </div>
      <div v-else class="mt-10px text-13px text-[#6b7280]">暂无输入设备信息</div>
    </div>

    <div class="rounded-10px border border-[#dbe2ea] p-14px">
      <div class="mb-8px text-15px font-600">输出设备兼容性</div>
      <div class="text-12px text-[#6b7280]">
        虚拟声卡播放端可作为“虚拟麦输出”，普通扬声器/耳机更适合作为“本地监听”。
      </div>
      <div v-if="catalog.outputs.length" class="mt-10px space-y-8px">
        <div
          v-for="device in catalog.outputs"
          :key="`output-${device.name}`"
          class="rounded-8px bg-[#f8fafc] px-10px py-8px text-13px"
        >
          <div class="font-600">{{ device.name }}</div>
          <div>
            状态: {{ device.compatible ? `支持 ${status.model_sample_rate}Hz 直连` : `不支持 ${status.model_sample_rate}Hz 直连` }}
          </div>
          <div>建议用途: {{ device.virtual_mic_candidate ? "优先作为虚拟麦输出" : "优先作为本地监听" }}</div>
          <div>采样格式: {{ device.supported_sample_formats.join(" / ") || "无" }}</div>
        </div>
      </div>
      <div v-else class="mt-10px text-13px text-[#6b7280]">暂无输出设备信息</div>
    </div>
  </div>

  <div class="mt-16px rounded-10px border border-[#dbe2ea] p-14px text-13px leading-22px">
    <div class="mb-8px text-15px font-600">运行指标</div>
    <div>输入缓冲溢出: {{ status.stats.input_overflows }}</div>
    <div>输出缓冲欠载: {{ status.stats.output_underflows }}</div>
    <div>推理帧数: {{ status.stats.inference_frames }}</div>
    <div>推理错误: {{ status.stats.inference_errors }}</div>
    <div>平均推理耗时: {{ status.stats.average_inference_ms.toFixed(2) }} ms</div>
    <div>最近一次推理耗时: {{ status.stats.last_inference_ms.toFixed(2) }} ms</div>
  </div>
</template>

<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { message } from "ant-design-vue";
import { onMounted, onUnmounted, reactive, ref, watch, computed } from "vue";
import { StorageKeys, StorageUtils } from "@/utils/storageUtils";

type AudioStatsSnapshot = {
  input_overflows: number;
  output_underflows: number;
  inference_frames: number;
  inference_errors: number;
  average_inference_ms: number;
  last_inference_ms: number;
};

type AudioStatus = {
  running: boolean;
  listening: boolean;
  sample_rate: number;
  model_sample_rate: number;
  input_device_name: string | null;
  virtual_output_device_name: string | null;
  monitor_output_device_name: string | null;
  input_channels: number | null;
  virtual_output_channels: number | null;
  monitor_output_channels: number | null;
  input_sample_format: string | null;
  virtual_output_sample_format: string | null;
  monitor_output_sample_format: string | null;
  stats: AudioStatsSnapshot;
};

type AudioDeviceInfo = {
  name: string;
  compatible: boolean;
  supported_sample_formats: string[];
  virtual_mic_candidate: boolean;
};

type AudioDeviceCatalog = {
  model_sample_rate: number;
  default_input: string | null;
  default_output: string | null;
  preferred_virtual_output: string | null;
  preferred_monitor_output: string | null;
  inputs: AudioDeviceInfo[];
  outputs: AudioDeviceInfo[];
};

type AudioRouteSelection = {
  inputDeviceName: string;
  virtualOutputDeviceName: string;
  monitorOutputDeviceName: string;
};

const loadSavedRouting = (): AudioRouteSelection => {
  const saved = StorageUtils.getItem(StorageKeys.AUDIO_ROUTE_SELECTION) || {};
  return {
    inputDeviceName: typeof saved.inputDeviceName === "string" ? saved.inputDeviceName : "",
    virtualOutputDeviceName:
      typeof saved.virtualOutputDeviceName === "string" ? saved.virtualOutputDeviceName : "",
    monitorOutputDeviceName:
      typeof saved.monitorOutputDeviceName === "string" ? saved.monitorOutputDeviceName : "",
  };
};

const busy = ref(false);
const loadingDevices = ref(false);
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
  default_input: null,
  default_output: null,
  preferred_virtual_output: null,
  preferred_monitor_output: null,
  inputs: [],
  outputs: [],
});
const routing = reactive<AudioRouteSelection>(loadSavedRouting());

let pollTimer: number | undefined;

const compatibleInputs = computed(() => catalog.inputs.filter((device) => device.compatible));
const compatibleOutputs = computed(() => catalog.outputs.filter((device) => device.compatible));
const monitorOutputOptions = computed(() =>
  compatibleOutputs.value.filter((device) => device.name !== routing.virtualOutputDeviceName),
);

const copyStatus = (next: AudioStatus) => {
  Object.assign(status, next, {
    stats: { ...next.stats },
  });
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

const hasCompatibleInput = (name: string | null | undefined) =>
  !!name && compatibleInputs.value.some((device) => device.name === name);

const hasCompatibleOutput = (name: string | null | undefined) =>
  !!name && compatibleOutputs.value.some((device) => device.name === name);

const firstCompatibleInputName = () => compatibleInputs.value[0]?.name || "";

const firstCompatibleOutputName = () => compatibleOutputs.value[0]?.name || "";

const chooseMonitorFallback = (excludedName: string) => {
  const preferred = compatibleOutputs.value.find(
    (device) => !device.virtual_mic_candidate && device.name !== excludedName,
  );
  if (preferred) {
    return preferred.name;
  }

  const anyCompatible = compatibleOutputs.value.find((device) => device.name !== excludedName);
  return anyCompatible?.name || "";
};

const reconcileRoutingSelection = () => {
  const nextInput = hasCompatibleInput(routing.inputDeviceName)
    ? routing.inputDeviceName
    : hasCompatibleInput(catalog.default_input)
      ? catalog.default_input!
      : firstCompatibleInputName();

  const nextVirtualOutput = hasCompatibleOutput(routing.virtualOutputDeviceName)
    ? routing.virtualOutputDeviceName
    : hasCompatibleOutput(catalog.preferred_virtual_output)
      ? catalog.preferred_virtual_output!
      : hasCompatibleOutput(catalog.default_output)
        ? catalog.default_output!
        : firstCompatibleOutputName();

  const preferredMonitor = hasCompatibleOutput(catalog.preferred_monitor_output) && catalog.preferred_monitor_output !== nextVirtualOutput
    ? catalog.preferred_monitor_output!
    : hasCompatibleOutput(catalog.default_output) && catalog.default_output !== nextVirtualOutput
      ? catalog.default_output!
      : chooseMonitorFallback(nextVirtualOutput);

  const nextMonitorOutput = hasCompatibleOutput(routing.monitorOutputDeviceName) && routing.monitorOutputDeviceName !== nextVirtualOutput
    ? routing.monitorOutputDeviceName
    : preferredMonitor;

  routing.inputDeviceName = nextInput;
  routing.virtualOutputDeviceName = nextVirtualOutput;
  routing.monitorOutputDeviceName = nextMonitorOutput;
};

const refreshStatus = async () => {
  const next = await invoke<AudioStatus>("get_audio_status");
  copyStatus(next);
};

const refreshDevices = async () => {
  loadingDevices.value = true;
  try {
    const next = await invoke<AudioDeviceCatalog>("list_audio_devices");
    copyCatalog(next);
    reconcileRoutingSelection();
  } catch (error) {
    message.error(formatError(error), 2);
  } finally {
    loadingDevices.value = false;
  }
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
      if (!routing.virtualOutputDeviceName) {
        throw new Error("请先选择虚拟麦输出设备");
      }
      if (routing.monitorOutputDeviceName && routing.monitorOutputDeviceName === routing.virtualOutputDeviceName) {
        throw new Error("监听设备不能和虚拟麦输出设备相同");
      }
      const res = await invoke<string>("start_noise_reduction", {
        inputDeviceName: routing.inputDeviceName || null,
        virtualOutputDeviceName: routing.virtualOutputDeviceName || null,
        monitorOutputDeviceName: routing.monitorOutputDeviceName || null,
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
  if (open && !routing.monitorOutputDeviceName) {
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
    if (routing.monitorOutputDeviceName === routing.virtualOutputDeviceName) {
      routing.monitorOutputDeviceName = "";
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
