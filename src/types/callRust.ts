export enum callRustType {
    AddSun = "AddSun", // 添加阳光
    GetSun = "GetSun", // 获取阳光值
    COOLING = "cooling", // 冷却修改

    NONE = "none", // 无操作
}



export enum cpalType {
    getDefaultAudioDevice = "getDefaultAudioDevice", // 获取默认音频设备
}

export enum windowPosType {
    /** 获取窗口句柄列表 */
    GETALLWINDOWS = "get_all_windows", 
    /** 设置窗口置顶 */
    SETWINDOWPOS = "set_window_pos_command", 
}   