import { createApp } from "vue";
import App from "./App.vue";

import "ant-design-vue/dist/reset.css";

createApp(App).mount("#app");

document.oncontextmenu = function () { // 关闭右键菜单
  return false;
};
