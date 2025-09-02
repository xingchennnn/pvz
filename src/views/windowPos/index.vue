<template>
  <div>
    <h2 class="text-lg font-bold mb-2">窗口列表</h2>
    <ul class="list-none p-0" role="listbox" aria-multiselectable="true">
      <li
        v-for="(win, index) in windows"
        :key="index"
        :class="[
          'border border-solid border-transparent bg-[var(--text-color)] c-[var(--bg-color)] p-1 mb-2 rounded flex flex-col cursor-pointer hover:bg-[#f0f0f0] transition-all duration-200',
          win.isActive ? 'bg-[#e6f4ff] border-[#1677ff] shadow-[0_0_0_2px_rgba(22,119,255,0.2)]' : ''
        ]"
        role="option"
        :aria-selected="win.isActive"
        @click="setActiveWindow(win, index)"
      >
        <span>标题: {{ win.title }}</span>
        <span>类名: {{ win.class_name }}</span>
        <span>句柄: {{ win.hwnd }}</span>
        <span v-if="win.isActive" class="mt-1 text-xs text-[#1677ff]">已选中</span>
      </li>
    </ul>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

const windows = ref<
  { title: string; class_name: string; hwnd: number; isActive: boolean }[]
>([]);

// 请求所有窗口
const getAllWindows = async () => {
  const JsonData = await invoke<string>("get_all_windows");
  let windows = JSON.parse(JsonData);

  windows = windows.filter((win: any) => {
    return ["IME", "MSCTFIME UI", "BroadcastListenerWindow"].includes(
      win.class_name
    ) === false;
  });

  return windows;
};

const setActiveWindow = (win: any, index: number) => {
  windows.value[index].isActive = !windows.value[index].isActive;
  invoke("set_window_pos_command", {
    hwnd: win.hwnd,
    insertAfter: win.isActive ? -1 : -2, // -1 置顶 -2 取消置顶
    x: 0,
    y: 0,
    cx: 0,
    cy: 0,
    uFlags: 3, // SWP_NOSIZE -  | SWP_NOMOVE
  });
};

onMounted(async () => {
  windows.value = await getAllWindows();
  console.log(windows.value);
});
</script>
