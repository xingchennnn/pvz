import { createMemoryHistory, createRouter } from "vue-router";

export const routes = [
  {
    path: "/",
    name: "Home",
    label: "首页",
    component: () => import("@/views/home/index.vue"),
  },
  {
    name: "About",
    label: "关于",
    path: "/about",
    component: () => import("@/views/about/index.vue"),
  },
  {
    name: "Cpal",
    label: "降噪",
    path: "/cpal",
    component: () => import("@/views/cpal/index.vue"),
  }
];


export const router = createRouter({
  history: createMemoryHistory(),
  routes,
});

// 路由守卫
router.beforeEach((to, from, next) => {
  console.log(`to: ${to.path} ->from: ${from.path}`);
  // 登录验证 todo
  next();
});
