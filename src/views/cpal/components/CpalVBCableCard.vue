<template>
  <section class="cpal-card cpal-card--accent">
    <div class="cpal-card__head">
      <div>
        <div class="cpal-card__title-row">
          <div class="cpal-card__title">VB-CABLE 集成</div>
          <span class="cpal-badge" :class="props.vbcable.installed ? 'is-success' : 'is-warning'">
            {{ props.vbcable.installed ? "已安装" : "未安装" }}
          </span>
        </div>
        <div class="cpal-card__subtitle">
          用于把降噪后的声音路由到通话软件，推荐把播放端作为虚拟麦输出。
        </div>
      </div>

      <div class="cpal-actions cpal-actions--compact">
        <a-button :loading="props.vbcableLoading" @click="emit('refresh-vbcable-status')">
          刷新 VB-CABLE
        </a-button>
        <a-button
          v-if="props.vbcable.installed"
          type="primary"
          :disabled="!props.vbcable.output_device_id"
          @click="emit('apply-vbcable-route')"
        >
          套用 VB-CABLE
        </a-button>
        <a-button v-if="props.vbcable.package_dir" @click="emit('open-vbcable-package-dir')">
          打开本地驱动包
        </a-button>
        <a-button
          v-if="props.vbcable.installed && props.vbcable.control_panel_path"
          @click="emit('open-vbcable-control-panel')"
        >
          控制面板
        </a-button>
        <a-button v-if="!props.vbcable.installed" @click="emit('open-vbcable-download-page')">
          官网下载
        </a-button>
      </div>
    </div>

    <div class="cpal-info-grid">
      <div class="cpal-info-item">
        <span class="cpal-info-label">播放端</span>
        <strong>{{ props.vbcable.output_device_name || "未检测到" }}</strong>
      </div>
      <div class="cpal-info-item">
        <span class="cpal-info-label">录音端</span>
        <strong>{{ props.vbcable.input_device_name || "未检测到" }}</strong>
      </div>
      <div class="cpal-info-item">
        <span class="cpal-info-label">本地驱动包</span>
        <strong>{{ props.vbcable.package_dir || "未发现" }}</strong>
      </div>
    </div>

    <div class="cpal-note-list">
      <div class="cpal-note">{{ props.vbcable.integration_notice }}</div>
      <div v-if="!props.vbcable.installed && props.vbcable.setup_path" class="cpal-note">
        安装提示: 打开驱动包目录后，以管理员身份运行 {{ props.vbcableSetupFileName }}。安装完成后请重启电脑，再回到这里刷新设备。
      </div>
      <div v-if="props.vbcable.installed && props.vbcable.input_device_name" class="cpal-note cpal-note--accent">
        通话软件麦克风请选择: {{ props.vbcable.input_device_name }}。
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import type { VBCableStatus } from "../types";

const props = defineProps<{
  vbcable: VBCableStatus;
  vbcableLoading: boolean;
  vbcableSetupFileName: string;
}>();

const emit = defineEmits<{
  (e: "refresh-vbcable-status"): void;
  (e: "apply-vbcable-route"): void;
  (e: "open-vbcable-package-dir"): void;
  (e: "open-vbcable-control-panel"): void;
  (e: "open-vbcable-download-page"): void;
}>();
</script>