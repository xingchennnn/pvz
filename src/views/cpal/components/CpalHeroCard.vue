<template>
  <section class="cpal-card cpal-hero">
    <div class="cpal-hero__content">
      <div class="cpal-kicker">Audio Pipeline</div>
      <div class="cpal-page__title">降噪设置</div>
      <div class="cpal-page__desc">
        麦克风输入 -> RNNoise 处理 -> 虚拟输出设备。
      </div>
      <div class="cpal-pills">
        <span class="cpal-pill" :class="props.status.running ? 'is-success' : 'is-idle'">
          引擎 {{ props.status.running ? "已启动" : "未启动" }}
        </span>
        <span class="cpal-pill" :class="props.status.listening ? 'is-primary' : 'is-idle'">
          监听 {{ props.status.listening ? "已打开" : "已关闭" }}
        </span>
        <span class="cpal-pill">目标 {{ props.status.model_sample_rate }} Hz</span>
        <span class="cpal-pill">当前 {{ props.status.sample_rate || "未建立" }}</span>
      </div>
    </div>

    <div class="cpal-actions">
      <a-button :loading="props.busy" type="primary" @click="emit('toggle-open-cpal')">
        {{ props.status.running ? "停止降噪" : "开始降噪" }}
      </a-button>
      <a-button
        :loading="props.busy"
        :disabled="!props.canToggleListener"
        @click="emit('toggle-listener')"
      >
        {{ props.status.listening ? "关闭监听" : "开启监听" }}
      </a-button>
      <a-button :loading="props.loadingDevices" @click="emit('refresh-devices')">刷新设备</a-button>
      <a-button :loading="props.busy" @click="emit('refresh-status')">刷新状态</a-button>
    </div>
  </section>
</template>

<script setup lang="ts">
import type { AudioStatus } from "../types";

const props = defineProps<{
  status: AudioStatus;
  busy: boolean;
  loadingDevices: boolean;
  canToggleListener: boolean;
}>();

const emit = defineEmits<{
  (e: "toggle-open-cpal"): void;
  (e: "toggle-listener"): void;
  (e: "refresh-devices"): void;
  (e: "refresh-status"): void;
}>();
</script>