<template>
  <div class="c-[var(--text-color)] text-18px font-600">降噪设置</div>
  <div class="mt-12px flex flex-wrap gap-12px">
    <a-button :loading="busy" @click="toggleOpenCpal" type="primary">
      {{ status.running ? "停止降噪" : "开始降噪" }}
    </a-button>
    <a-button
      :loading="busy"
      @click="toggleListener(!status.listening)"
      :disabled="!status.running"
    >
      {{ status.listening ? "关闭监听" : "开启监听" }}
    </a-button>
    <a-button :loading="loadingDevices" @click="refreshDevices">
      刷新设备
    </a-button>
    <a-button :loading="busy" @click="refreshStatus">刷新状态</a-button>
  </div>

  <div class="mt-16px rounded-10px bg-[#f5f7fa] p-14px text-14px leading-22px text-[#1f2937]">
    <div>引擎状态: {{ status.running ? "已启动" : "未启动" }}</div>
    <div>监听状态: {{ status.listening ? "处理中" : "未处理" }}</div>
    <div>模型目标采样率: {{ status.model_sample_rate }} Hz</div>
    <div>当前采样率: {{ status.sample_rate || "未建立链路" }}</div>
    <div>输入设备: {{ status.input_device_name || "未绑定" }}</div>
    <div>输出设备: {{ status.output_device_name || "未绑定" }}</div>
    <div>
      输入格式: {{ status.input_sample_format || "-" }} / {{ status.input_channels || "-" }} 声道
    </div>
    <div>
      输出格式: {{ status.output_sample_format || "-" }} / {{ status.output_channels || "-" }} 声道
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
            状态: {{ device.compatible ? "支持 16k 直连" : "不支持 16k 直连" }}
          </div>
          <div>采样格式: {{ device.supported_sample_formats.join(" / ") || "无" }}</div>
        </div>
      </div>
      <div v-else class="mt-10px text-13px text-[#6b7280]">暂无输入设备信息</div>
    </div>

    <div class="rounded-10px border border-[#dbe2ea] p-14px">
      <div class="mb-8px text-15px font-600">输出设备兼容性</div>
      <div class="text-12px text-[#6b7280]">
        后续接入虚拟设备时，这里会用来判断是否可作为目标输出。
      </div>
      <div v-if="catalog.outputs.length" class="mt-10px space-y-8px">
        <div
          v-for="device in catalog.outputs"
          :key="`output-${device.name}`"
          class="rounded-8px bg-[#f8fafc] px-10px py-8px text-13px"
        >
          <div class="font-600">{{ device.name }}</div>
          <div>
            状态: {{ device.compatible ? "支持 16k 直连" : "不支持 16k 直连" }}
          </div>
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
import { onMounted, onUnmounted, reactive, ref } from "vue";

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
  output_device_name: string | null;
  input_channels: number | null;
  output_channels: number | null;
  input_sample_format: string | null;
  output_sample_format: string | null;
  stats: AudioStatsSnapshot;
};

type AudioDeviceInfo = {
  name: string;
  compatible: boolean;
  supported_sample_formats: string[];
};

type AudioDeviceCatalog = {
  model_sample_rate: number;
  default_input: string | null;
  default_output: string | null;
  inputs: AudioDeviceInfo[];
  outputs: AudioDeviceInfo[];
};

const busy = ref(false);
const loadingDevices = ref(false);
const status = reactive<AudioStatus>({
  running: false,
  listening: false,
  sample_rate: 0,
  model_sample_rate: 16000,
  input_device_name: null,
  output_device_name: null,
  input_channels: null,
  output_channels: null,
  input_sample_format: null,
  output_sample_format: null,
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
  model_sample_rate: 16000,
  default_input: null,
  default_output: null,
  inputs: [],
  outputs: [],
});

let pollTimer: number | undefined;

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

const refreshStatus = async () => {
  const next = await invoke<AudioStatus>("get_audio_status");
  copyStatus(next);
};

const refreshDevices = async () => {
  loadingDevices.value = true;
  try {
    const next = await invoke<AudioDeviceCatalog>("list_audio_devices");
    copyCatalog(next);
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
      const res = await invoke<string>("start_noise_reduction");
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
