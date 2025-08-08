import { createApp } from "vue";
import App from "./App.vue";
import { createPinia } from 'pinia'; // 引入pinia
import {router} from "./routers/index";

import "ant-design-vue/dist/reset.css";
import 'virtual:uno.css'; // 引入Uno.css


const pinia = createPinia()

createApp(App).use(pinia).use(router).mount("#app");


// if(process.env.NODE_ENV === 'production'){
//   document.oncontextmenu = function () { // 关闭右键菜单
//     return false;
//   };
// }

import './assets/scss/main.css' // 引入全局样式文件


