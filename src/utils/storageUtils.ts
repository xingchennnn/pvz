// 设置枚举类型
export enum StorageKeys {
    /**用户信息 */
    USER_INFO = 'USER_INFO',
    /**页面配置 暗色模式 */
    PAGE_CONFIG_DARK_MODE = 'PAGE_CONFIG_DARK_MODE',
}


export class StorageUtils {
    /**
     * 获取存储的值
     * @param key 键名
     * @returns 存储的值或 null
     */
    static getItem(key: StorageKeys): any {
        const value = localStorage.getItem(key);
        return value ? JSON.parse(value) : null;
    }

    /**
     * 设置存储的值
     * @param key 键名
     * @param value 值
     */
    static setItem(key: StorageKeys, value: any): void {
        localStorage.setItem(key, JSON.stringify(value));
    }

    /**
     * 删除存储的值
     * @param key 键名
     */
    static removeItem(key: StorageKeys): void {
        localStorage.removeItem(key);
    }
}