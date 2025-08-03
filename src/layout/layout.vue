<template>
  <a-layout class="h-100vh">
    <a-layout-sider v-model:collapsed="collapsed" :trigger="null" collapsible
      :theme="themeStore.isDarkMode ? 'dark' : 'light'">
      <div class="logo flex justify-start items-center h-[var(--logo-height)] px-2">
        <img src="@/assets/trans_bg_.png" alt="logo" class="h-100%" />
        <span class="flex  fw-bold c-[var(--text-color)]">Star</span>
      </div>
      <a-menu v-model:selectedKeys="selectedKeys" :theme="themeStore.isDarkMode ? 'dark' : 'light'" mode="inline" class="!h-[calc(100%-var(--logo-height))]">
        <a-menu-item key="/" @click="() => router.push('/')" >
          <user-outlined />
          <span>植物大战僵尸</span>
        </a-menu-item>
        <a-menu-item key="/cpal" @click="() => router.push('/cpal')" >
          <video-camera-outlined />
          <span>降噪</span>
        </a-menu-item>
        <a-menu-item key="/about" @click="() => router.push('/about')">
          <upload-outlined />
          <span>关于</span>
        </a-menu-item>
      </a-menu>
    </a-layout-sider>
    <a-layout class="p-0">
      <a-layout-header class="h-24px flex justify-between items-center bg-[var(--bg-color)]" >
        <div class="header-left">
          <i class="color-[var(--text-color)]">
            <menu-unfold-outlined v-if="collapsed" class="trigger " @click="() => (collapsed = !collapsed)" />
            <menu-fold-outlined v-else class="trigger" @click="() => (collapsed = !collapsed)"  />
          </i>
        </div>
        <div class="header-right">
          <a-switch :checked="themeStore.isDarkMode" @change="themeChange">
            <template #checkedChildren>
              <i class="c-[#FF0]">
                <svg xmlns="http://www.w3.org/2000/svg" role="img" width="1em" height="1em" viewBox="0 0 64 64"
                  class="iconify iconify--emojione-monotone">
                  <path fill="currentColor"
                    d="m20.52 59.717l7.027-7.2a20.9 20.9 0 0 1-6.904-2.87zM43.48 4.284l-7.025 7.199a20.9 20.9 0 0 1 6.904 2.871zm-31.996 32.17l-7.201 7.025l10.07-.122a20.9 20.9 0 0 1-2.869-6.903m41.032-8.907l7.201-7.027l-10.07.123a20.9 20.9 0 0 1 2.869 6.904m-38.162-6.905l-10.07-.123l7.201 7.027a20.8 20.8 0 0 1 2.869-6.904m35.292 22.716l10.07.122l-7.201-7.026a20.8 20.8 0 0 1-2.869 6.904M27.547 11.483l-7.027-7.2l.123 10.07a20.9 20.9 0 0 1 6.904-2.87m8.906 41.034l7.027 7.199l-.123-10.069a20.9 20.9 0 0 1-6.904 2.87m-21.701-8.555l-3.967 9.251l9.252-3.965a21.1 21.1 0 0 1-5.285-5.286m34.496-23.923l3.965-9.252l-9.25 3.965a21.1 21.1 0 0 1 5.285 5.287M11 32c0-1.278.133-2.524.352-3.741L2 31.999l9.352 3.74A21 21 0 0 1 11 32m51 0l-9.352-3.741C52.867 29.476 53 30.722 53 32s-.133 2.525-.352 3.741zM20.039 14.751l-9.252-3.965l3.965 9.252a21.2 21.2 0 0 1 5.287-5.287m23.922 34.497l9.252 3.965l-3.965-9.251a21.1 21.1 0 0 1-5.287 5.286M35.74 11.352L32 2l-3.74 9.352C29.475 11.133 30.721 11 32 11s2.525.133 3.74.352m-7.48 41.296L32 62l3.74-9.352c-1.215.219-2.461.352-3.74.352s-2.525-.133-3.74-.352">
                  </path>
                  <circle cx="32" cy="32" r="19" fill="currentColor"></circle>
                </svg>
              </i>
            </template>
            <template #unCheckedChildren>
              <i class="c-[#FF0]">
                <svg xmlns="http://www.w3.org/2000/svg" role="img" width="1em" height="1em" viewBox="0 0 64 64"
                  class="iconify iconify--emojione-monotone">
                  <path fill="currentColor"
                    d="M43.139 2a29.9 29.9 0 0 1 5.121 16.756c0 16.701-13.686 30.24-30.57 30.24a30.66 30.66 0 0 1-15.689-4.285C7.209 54.963 17.93 62 30.318 62C47.816 62 62 47.969 62 30.66C62 17.867 54.246 6.871 43.139 2">
                  </path>
                </svg>
              </i>
            </template>
          </a-switch>
        </div>
      </a-layout-header>
      <a-layout-content class="p-1 min-h-280px min-w-280px bg-[var(--bg-color)]">
        <RouterView />
      </a-layout-content>
    </a-layout>
  </a-layout>
</template>
<script lang="ts" setup>
import { onMounted, ref } from 'vue';
import { RouterView, useRouter } from "vue-router";
import {
  UserOutlined,
  VideoCameraOutlined,
  UploadOutlined,
  MenuUnfoldOutlined,
  MenuFoldOutlined,
} from '@ant-design/icons-vue';
import { router as  routerList} from '@/routers';
import { useThemeStore } from '@/store/theme';

const themeStore = useThemeStore();
const selectedKeys = ref<string[]>(['/']); // 选中的菜单项
const collapsed = ref<boolean>(false); // 侧边栏是否收起
const router = useRouter();

// 监听主题色变化
const themeChange = (e: any) => {
  // console.log(`output->${e}`);
  themeStore.toggleDarkMode();
};


onMounted(() => {
  // 监听路由变化
  console.log('onMounted', routerList);

});

</script>
