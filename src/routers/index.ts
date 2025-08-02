import { createMemoryHistory, createRouter } from "vue-router";

const routes = [
  {
    path: "/",
    name: "Home",
    component: () => import("@/views/home/index.vue"),
  },
  {
    path: "/about",
    name: "About",
    component: () => import("@/views/about/index.vue"),
  },
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
