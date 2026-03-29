<template>
  <section class="cpal-grid cpal-grid--two">
    <article class="cpal-card">
      <div class="cpal-card__title">输入设备兼容性</div>
      <div class="cpal-card__subtitle">
        当前实现只接受可直接工作在 {{ props.modelSampleRate }} Hz 的设备。
      </div>

      <div v-if="props.inputItems.length" class="device-list">
        <div v-for="item in props.inputItems" :key="item.id" class="device-item">
          <div class="device-item__head">
            <div class="device-item__title">{{ item.label }}</div>
            <span class="cpal-badge" :class="`is-${item.badgeVariant}`">{{ item.badgeLabel }}</span>
          </div>
          <div v-for="line in item.lines" :key="line" class="device-item__meta">{{ line }}</div>
        </div>
      </div>
      <div v-else class="cpal-empty">暂无输入设备信息</div>
    </article>

    <article class="cpal-card">
      <div class="cpal-card__title">输出设备兼容性</div>
      <div class="cpal-card__subtitle">
        虚拟声卡播放端适合作为“虚拟麦输出”，普通扬声器/耳机更适合作为“本地监听”。
      </div>

      <div v-if="props.outputItems.length" class="device-list">
        <div v-for="item in props.outputItems" :key="item.id" class="device-item">
          <div class="device-item__head">
            <div class="device-item__title">{{ item.label }}</div>
            <span class="cpal-badge" :class="`is-${item.badgeVariant}`">{{ item.badgeLabel }}</span>
          </div>
          <div v-for="line in item.lines" :key="line" class="device-item__meta">{{ line }}</div>
        </div>
      </div>
      <div v-else class="cpal-empty">暂无输出设备信息</div>
    </article>
  </section>
</template>

<script setup lang="ts">
import type { AudioDevicePanelItem } from "../types";

const props = defineProps<{
  modelSampleRate: number;
  inputItems: AudioDevicePanelItem[];
  outputItems: AudioDevicePanelItem[];
}>();
</script>