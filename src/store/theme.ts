import { StorageKeys, StorageUtils } from '@/utils/storageUtils'
import { defineStore } from 'pinia'

//  `defineStore()` 的返回值的命名是自由的
// 但最好含有 store 的名字，且以 `use` 开头，以 `Store` 结尾。
// (比如 `useUserStore`，`useCartStore`，`useProductStore`)
// 第一个参数是你的应用中 Store 的唯一 ID。
export const useThemeStore = defineStore('theme', {
    // 其他配置...
    // 这里可以添加 getters、actions、mutations 等等
    state: () => ({
        checked3: StorageUtils.getItem(StorageKeys.PAGE_CONFIG_DARK_MODE) || false, // 是否开启暗黑模式
    }),
    getters: {
        isDarkMode: (state) => state.checked3, // 获取暗黑模式状态
    },
    actions: {
        // 这里可以设置 actions
        toggleDarkMode() {
            // 切换状态
            this.checked3 = !this.checked3
            document.body.classList.toggle('dark-theme')
            // 持久化到 localStorage
            StorageUtils.setItem(StorageKeys.PAGE_CONFIG_DARK_MODE, this.checked3)
        }
    }
})