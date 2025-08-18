import { createWebHashHistory, createRouter } from "vue-router";
import {
  UserOutlined,
  VideoCameraOutlined,
  UploadOutlined,
  HomeOutlined
} from "@ant-design/icons-vue";

export const routes = [
  {
    path: "/",
    name: "Home",
    label: "首页",
    icon: HomeOutlined,
    component: () => import("@/views/home/index.vue"),
  },
  {
    name: "文章",
    label: "文章",
    path: "/article",
    icon: UploadOutlined,
    component: () => import("@/views/article/index.vue"),
  },
  {
    name: "文章详情",
    label: "文章详情",
    path: "/article/detail",
    icon: UploadOutlined,
    component: () => import("@/views/article/detail.vue"),
    meta: {
      hideLayout: true, // 隐藏layout
      hideMenu: true,  // 隐藏菜单
    },
  },

  {
    name: "Cpal",
    label: "降噪",
    path: "/cpal",
    icon: UploadOutlined,
    component: () => import("@/views/cpal/index.vue"),
  },
  {
    name: "降噪详情",
    label: "降噪详情",
    path: "/cpal/detail",
    icon: UploadOutlined,
    component: () => import("@/views/cpal/detail.vue"),
    meta: {
      hideLayout: true, // 隐藏layout
      hideMenu: true,  // 隐藏菜单
    },
  },
  {
    name: "pvz",
    label: "萌新",
    path: "/pvz",
    icon: VideoCameraOutlined,
    component: () => import("@/views/pvz/index.vue"),
  },
  {
    name: "About",
    label: "关于",
    path: "/about",
    icon: UserOutlined,
    component: () => import("@/views/about/index.vue"),
  },
];


export const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

// 路由守卫
router.beforeEach((to, from, next) => {
  console.log(`to: ${to.path} ->from: ${from.path}`);
  // 登录验证 todo
  next();
});
