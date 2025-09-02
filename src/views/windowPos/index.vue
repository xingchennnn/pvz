<template>
  <div>
    <h2 class="text-lg font-bold mb-2">窗口列表</h2>
    <a-card class="w-full  mb-3">
      <div class="flex justify-start items-center gap-1">
        <a-input
          v-model:value="searchText"
          placeholder="搜索窗口"
          clearable
          class="w-240px"
        ></a-input>
        <a-button type="primary" @click="searchWindows">搜索</a-button>
        <a-button  @click="refreshWindows">刷新</a-button>
      </div>
    </a-card>
    <ul class="list-none p-0" role="listbox" aria-multiselectable="true">
      <li
        v-for="(win, index) in windows"
        :key="index"
        :class="[
          'border border-solid border-transparent bg-[var(--bg-color)] c-[var(--text-color)] p-1 mb-2 rounded flex flex-col cursor-pointer hover:bg-[var(--hover-color)] transition-all duration-200',
          win.isActive
            ? 'bg-[#e6f4ff] border-[#1677ff] shadow-[0_0_0_2px_rgba(22,119,255,0.2)]'
            : '',
        ]"
        role="option"
        :aria-selected="win.isActive"
        @click="setActiveWindow(win, index)"
      >
        <span>标题: {{ win.title }}</span>
        <span>类名: {{ win.class_name }}</span>
        <span>句柄: {{ win.hwnd }}</span>
        <span v-if="win.isActive" class="mt-1 text-xs text-[#1677ff]"
          >已选中</span
        >
      </li>
    </ul>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
// import { webviewWindow } from "@tauri-apps/api";
import { windowPosType } from "@/types/callRust";

const windows = ref<
  { title: string; class_name: string; hwnd: number; isActive: boolean }[]
>([]);

// 请求所有窗口
const getAllWindows = async () => {
  const JsonData = await invoke<string>(windowPosType.GETALLWINDOWS);
  let windows_copy = JSON.parse(JsonData);

  windows_copy = windows_copy.filter((win: any) => {
    return (
      ["IME", "MSCTFIME UI", "BroadcastListenerWindow",'Window'].includes(
        win.class_name
      ) === false
    );
  });
  return windows_copy;
};

const searchText = ref<string>("");
/**搜索窗口 */
const searchWindows = async () => {
  let _windows = await getAllWindows();
  windows.value = _windows.filter((win: any) =>
    win.title.includes(searchText.value)
  );
};
/**刷新窗口列表 */
const refreshWindows = async () => {
  searchText.value = "";
  windows.value = await getAllWindows();
};

/**设置窗口置顶 */
const setActiveWindow = async (win: any, index: number) => {
  windows.value[index].isActive = !windows.value[index].isActive;
  await invoke(windowPosType.SETWINDOWPOS, {
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
});
</script>
