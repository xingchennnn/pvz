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
        isDarkMode: (state) => {
            return state.checked3
        }, // 获取暗黑模式状态
    },
    actions: {
        init(){
            // 初始化时，从 localStorage 中获取是否开启暗黑模式
            const checked3 = StorageUtils.getItem(StorageKeys.PAGE_CONFIG_DARK_MODE)
            if (checked3) {
                this.checked3 = checked3
                document.body.classList.add('dark-theme')
            }
        },
        // 这里可以设置 actions
        toggleDarkMode() {
            // 切换状态
            this.checked3 = !this.checked3
            // console.log('切换暗黑模式', this.checked3);

            if (this.checked3) {
                document.body.classList.add('dark-theme')
            } else {
                document.body.classList.remove('dark-theme')
            }
            // 持久化到 localStorage
            StorageUtils.setItem(StorageKeys.PAGE_CONFIG_DARK_MODE, this.checked3)
        },
        // 设置暗黑模式
        setDarkMode(value: boolean) {

            // 如果传入的 value 与当前状态一致，则不执行任何操作
            if (value === this.checked3) return;
            // 设置状态
            this.checked3 = value
            // console.log('设置暗黑模式', this.checked3);

            if (this.checked3) {
                document.body.classList.add('dark-theme')
            } else {
                document.body.classList.remove('dark-theme')
            }
            // 持久化到 localStorage
            StorageUtils.setItem(StorageKeys.PAGE_CONFIG_DARK_MODE, this.checked3)
        },
    }
})