import { createApp } from "vue";
import App from "./App.vue";
import { createPinia } from 'pinia'; // 引入pinia
import {router} from "./routers/index";

import "ant-design-vue/dist/reset.css";
import 'virtual:uno.css'; // 引入Uno.css

const pinia = createPinia()

createApp(App).use(pinia).use(router).mount("#app");

document.oncontextmenu = function () { // 关闭右键菜单
  return false;
};



