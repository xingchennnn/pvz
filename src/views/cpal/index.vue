<template>
  <div class="c-[var(--text-color)]">降噪设置</div>
  <div>
    <a-button v-if="!isopen" @click="toogleOpenCpal" type="primary"
      >开始降噪</a-button
    >
    <a-button v-else @click="toogleOpenCpal">停止降噪</a-button>
    <a-button
      v-if="!isListenerOpen"
      @click="openListener(true)"
      :class="`ml-10px ${isopen ? '' : '!bg-gray-200'}`"
      type="primary"
      :disabled="!isopen"
      >开启监听</a-button
    >
    <a-button v-else @click="openListener(false)" class="ml-10px"
      >关闭监听</a-button
    >
    <a-button @click="createWindow">打开一个新窗口</a-button>
  </div>
</template>

<script setup lang="ts">
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { invoke } from "@tauri-apps/api/core";
// import { Window } from "@tauri-apps/api/window";
import { message } from "ant-design-vue";
import { ref } from "vue";

const isopen = ref(false); // 是否开启降噪
const isListenerOpen = ref(false); // 是否开启监听

const toogleOpenCpal = () => {
  if (isopen.value) {
    isListenerOpen.value && openListener(false); // 关闭监听
    stopInCpal(); // 停止降噪
  } else {
    startInCpal();
  }
  isopen.value = !isopen.value;
};

const startInCpal = async () => {
  const res = await invoke<string>("start_noise_reduction");
  // console.log("开始降噪:", res);
  message.info(res, 1);
};

const stopInCpal = async () => {
  const res = await invoke<string>("stop_noise_reduction");
  message.info(res, 1);
};

const openListener = async (open: boolean) => {
  if (open) {
    const res = await invoke<string>("open_listener");
    isListenerOpen.value = true;
    message.info(res, 1);
  } else {
    const res = await invoke<string>("close_listener");
    isListenerOpen.value = false;
    message.info(res, 1);
  }
};

/**新窗口 */
const createWindow = () => {
  const Window1 = new WebviewWindow("detail", {
    url: "#/cpal/detail",
    title: "详情",
    width: 800,
    height: 600,
    center: true,
    // resizable: true,
    // alwaysOnTop: true,
    // transparent: true,
    // hiddenTitle: true,
    // maximized: true,
    // visible: false,
    // skipTaskbar: false,
  });
  Window1.show();
  Window1.once("tauri://created", () => {
    console.log("new window created");
  });

  Window1.once("tauri://error", function (e) {
    // an error happened creating the window
    console.error(e);
  });
};
</script>
