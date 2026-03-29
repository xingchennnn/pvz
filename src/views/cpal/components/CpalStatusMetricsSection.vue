<template>
  <section class="cpal-grid cpal-grid--two">
    <article class="cpal-card">
      <div class="cpal-card__title-row">
        <div class="cpal-card__title">链路状态</div>
        <span class="cpal-soft-tag">{{ props.status.sample_rate || "未建立链路" }} {{ props.status.sample_rate ? "Hz" : "" }}</span>
      </div>

      <div class="cpal-fact-grid">
        <div class="cpal-fact-item">
          <span>引擎状态</span>
          <strong>{{ props.status.running ? "已启动" : "未启动" }}</strong>
        </div>
        <div class="cpal-fact-item">
          <span>监听状态</span>
          <strong>{{ props.status.listening ? "已打开" : "已关闭" }}</strong>
        </div>
        <div class="cpal-fact-item">
          <span>输入设备</span>
          <strong>{{ props.status.input_device_name || "未绑定" }}</strong>
        </div>
        <div class="cpal-fact-item">
          <span>虚拟麦输出</span>
          <strong>{{ props.status.virtual_output_device_name || "未绑定" }}</strong>
        </div>
        <div class="cpal-fact-item">
          <span>监听设备</span>
          <strong>{{ props.status.monitor_output_device_name || "未配置" }}</strong>
        </div>
        <div class="cpal-fact-item">
          <span>目标采样率</span>
          <strong>{{ props.status.model_sample_rate }} Hz</strong>
        </div>
        <div class="cpal-fact-item">
          <span>输入格式</span>
          <strong>{{ props.status.input_sample_format || "-" }} / {{ props.status.input_channels || "-" }} 声道</strong>
        </div>
        <div class="cpal-fact-item">
          <span>虚拟麦格式</span>
          <strong>{{ props.status.virtual_output_sample_format || "-" }} / {{ props.status.virtual_output_channels || "-" }} 声道</strong>
        </div>
        <div class="cpal-fact-item">
          <span>监听格式</span>
          <strong>{{ props.status.monitor_output_sample_format || "-" }} / {{ props.status.monitor_output_channels || "-" }} 声道</strong>
        </div>
      </div>
    </article>

    <article class="cpal-card">
      <div class="cpal-card__title">运行指标</div>
      <div class="cpal-metric-grid">
        <div class="cpal-metric-item">
          <span>输入缓冲溢出</span>
          <strong>{{ props.status.stats.input_overflows }}</strong>
        </div>
        <div class="cpal-metric-item">
          <span>输出缓冲欠载</span>
          <strong>{{ props.status.stats.output_underflows }}</strong>
        </div>
        <div class="cpal-metric-item">
          <span>推理帧数</span>
          <strong>{{ props.status.stats.inference_frames }}</strong>
        </div>
        <div class="cpal-metric-item">
          <span>推理错误</span>
          <strong>{{ props.status.stats.inference_errors }}</strong>
        </div>
        <div class="cpal-metric-item">
          <span>平均推理耗时</span>
          <strong>{{ props.status.stats.average_inference_ms.toFixed(2) }} ms</strong>
        </div>
        <div class="cpal-metric-item">
          <span>最近一次推理耗时</span>
          <strong>{{ props.status.stats.last_inference_ms.toFixed(2) }} ms</strong>
        </div>
      </div>
    </article>
  </section>
</template>

<script setup lang="ts">
import type { AudioStatus } from "../types";

const props = defineProps<{
  status: AudioStatus;
}>();
</script>