import { invoke } from "@tauri-apps/api/core";
import { callRustType, cpalType } from "@/types/callRust";
import { message } from "ant-design-vue";

let lastErrorMsg = "";
let lastErrorTime = 0;
const ERROR_INTERVAL = 10000; // 10秒内只弹一次

export function callRust(types: callRustType | cpalType, text: any): Promise<string> {
  return new Promise(async (resolve, reject) => {
    let result: string = await invoke("call_rust", { types, text });

    if (result.startsWith("错误")) {
      const now = Date.now();
      if (
        result !== lastErrorMsg || // 错误内容不同则弹
        now - lastErrorTime > ERROR_INTERVAL // 或距离上次弹窗已过间隔
      ) {
        message.error('请检查游戏是否正常运行');
        lastErrorMsg = result;
        lastErrorTime = now;
      }
      reject(result);
    } else {
      resolve(result);
    }
  });
}


