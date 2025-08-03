<template>
        <div class="c-[var(--text-color)]"> 降噪设置 </div>
        <div>
                <a-button @click="startInCpal" type="primary">开始降噪</a-button>
                <a-button @click="stopInCpal" class="ml-10px">停止降噪</a-button>
        </div>
</template>

<script setup lang='ts'>
import { invoke } from "@tauri-apps/api/core";
import { message } from "ant-design-vue";


const startInCpal = async () => {
        const res = await invoke<string>("start_noise_reduction");
        // console.log("开始降噪:", res);
        message.info(res);
};

const stopInCpal = async () => {
        const res = await invoke<string>("stop_noise_reduction");
        // console.log("停止降噪:", res);
        message.info(res);
};
</script>