<template>
  <section class="cpal-grid cpal-grid--three">
    <article class="cpal-card setting-card">
      <div class="cpal-card__title">输入麦克风</div>
      <div class="cpal-card__subtitle">原始麦克风采集源，修改后需要重新启动降噪引擎。</div>
      <a-select v-model:value="inputDeviceIdModel" class="w-full" placeholder="选择输入麦克风">
        <a-select-option
          v-for="option in props.inputOptions"
          :key="`route-input-${option.id}`"
          :value="option.id"
        >
          {{ option.label }}
        </a-select-option>
      </a-select>
    </article>

    <article class="cpal-card setting-card">
      <div class="cpal-card__title-row">
        <div class="cpal-card__title">虚拟麦输出</div>
        <span class="cpal-soft-tag">推荐 VB-CABLE</span>
      </div>
      <div class="cpal-card__subtitle">
        处理后的声音会持续送到这里。若使用 VB-CABLE，应选择播放端 CABLE Input。
      </div>
      <a-select
        v-model:value="virtualOutputDeviceIdModel"
        class="w-full"
        placeholder="选择虚拟麦输出设备"
      >
        <a-select-option
          v-for="option in props.virtualOutputOptions"
          :key="`route-virtual-${option.id}`"
          :value="option.id"
        >
          {{ option.label }}{{ option.recommended ? " · 推荐" : "" }}
        </a-select-option>
      </a-select>
    </article>

    <article class="cpal-card setting-card">
      <div class="cpal-card__title">本地监听设备</div>
      <div class="cpal-card__subtitle">“开启监听”只控制这一路，不影响虚拟麦输出。</div>
      <a-select
        v-model:value="monitorOutputDeviceIdModel"
        class="w-full"
        placeholder="可选，不监听可留空"
      >
        <a-select-option value="">不启用本地监听</a-select-option>
        <a-select-option
          v-for="option in props.monitorOutputOptions"
          :key="`route-monitor-${option.id}`"
          :value="option.id"
        >
          {{ option.label }}
        </a-select-option>
      </a-select>
    </article>
  </section>
</template>

<script setup lang="ts">
import { computed } from "vue";
import type { AudioSelectOption } from "../types";

const props = defineProps<{
  inputDeviceId: string;
  virtualOutputDeviceId: string;
  monitorOutputDeviceId: string;
  inputOptions: AudioSelectOption[];
  virtualOutputOptions: AudioSelectOption[];
  monitorOutputOptions: AudioSelectOption[];
}>();

const emit = defineEmits<{
  (e: "change-input-device-id", value: string): void;
  (e: "change-virtual-output-device-id", value: string): void;
  (e: "change-monitor-output-device-id", value: string): void;
}>();

const inputDeviceIdModel = computed({
  get: () => props.inputDeviceId,
  set: (value: string) => emit("change-input-device-id", value),
});

const virtualOutputDeviceIdModel = computed({
  get: () => props.virtualOutputDeviceId,
  set: (value: string) => emit("change-virtual-output-device-id", value),
});

const monitorOutputDeviceIdModel = computed({
  get: () => props.monitorOutputDeviceId,
  set: (value: string) => emit("change-monitor-output-device-id", value),
});
</script>