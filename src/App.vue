<template>
    <div>
        <!-- <RouterView /> -->
        <ConfigProvider :locale="zhCN" >
            <layout />
        </ConfigProvider>

    </div>
</template>

<script lang="ts" setup>
// import { RouterView } from 'vue-router'
import layout from './layout/layout.vue';
import { ConfigProvider } from 'ant-design-vue'
import zhCN from 'ant-design-vue/es/locale/zh_CN'

// const themes = {
//     algorithm: theme.darkAlgorithm,
//     components: {
//         Layout: {
//             siderBg: '#141414',
//             headerBg: '#141414',
//             bodyBg: "#141414",
//             colorText: "#fff"
//         },
//         // Menu: {
//         //     // darkItemColor: "#fff",
//         //     darkItemBg: '#141414',
//         //     // darkItemHoverColor: "#fff",
//         //     darkItemHoverBg: "#333",
//         //     itemActiveBg: "#00513b"
//         // }
//     },
//     token: {
//         // colorPrimary: '#00a47a',
//         // colorPrimaryBg: "#141414", // '#2c2c2c',
//         // colorText: "rgba(255,255,255,0.65)",
//         // colorTextDescription: "rgba(255,255,255,0.65)"
//     },
// }
</script>
