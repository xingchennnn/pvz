import { invoke } from "@tauri-apps/api/core";
import { callRustType } from "@/types/callRust";
import { message } from "ant-design-vue";

export function callRust(types: callRustType, text: any): Promise<string> {
  return new Promise(async (resolve, reject) => {
    let result: string = await invoke("call_rust", { types, text });

    if (result.startsWith("错误")) {
      message.error(result);
      reject(result);
    } else {
      resolve(result);
    }
  });
}
