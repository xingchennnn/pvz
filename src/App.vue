<template>
    <div>
        <!-- <RouterView /> -->
        <ConfigProvider :locale="zhCN" :theme="{ algorithm: isDarkMode ? darkAlgorithm : defaultAlgorithm }">
            <layout />
        </ConfigProvider>

    </div>
</template>

<script lang="ts" setup>
// import { RouterView } from 'vue-router'
import layout from './layout/layout.vue';
import { ConfigProvider ,theme  } from 'ant-design-vue'
import zhCN from 'ant-design-vue/es/locale/zh_CN'
import { useThemeStore } from '@/store/theme'
import { computed } from 'vue';

const { defaultAlgorithm, darkAlgorithm } = theme;

const useTheme = useThemeStore()
const isDarkMode = computed(() => useTheme.isDarkMode)
</script>
