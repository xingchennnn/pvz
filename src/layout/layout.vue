<template>
  <a-layout class="h-100vh">
    <a-layout-sider v-model:collapsed="collapsed" :trigger="null" collapsible theme="light">
      <div class="logo flex justify-start items-center" >
        <img src="@/assets/original.png" alt="logo" class="w-10 h-10"/>
        <span class="flex  fw-bold">DMI</span>
      </div>
      <a-menu v-model:selectedKeys="selectedKeys" theme="light" mode="inline">
        <a-menu-item key="/" @click="() => router.push('/')">
          <user-outlined />
          <span>植物大战僵尸</span>
        </a-menu-item>
        <a-menu-item key="2">
          <video-camera-outlined />
          <span>降噪</span>
        </a-menu-item>
        <a-menu-item key="/about" @click="() => router.push('/about')">
          <upload-outlined />
          <span>关于</span>
        </a-menu-item>
      </a-menu>
    </a-layout-sider>
    <a-layout theme="light">
      <a-layout-header  class="bg-#fff p-0"  >
        <menu-unfold-outlined
          v-if="collapsed"
          class="trigger"
          @click="() => (collapsed = !collapsed)"
        />
        <menu-fold-outlined v-else class="trigger" @click="() => (collapsed = !collapsed)" />
      </a-layout-header>
      <a-layout-content
        class="bg-#fff m-1 p-4 min-h-280px min-w-280px"
      >
        <RouterView />
      </a-layout-content>
    </a-layout>
  </a-layout>
</template>
<script lang="ts" setup>
import { onMounted, ref } from 'vue';
import { RouterView , useRouter } from "vue-router";
import {
  UserOutlined,
  VideoCameraOutlined,
  UploadOutlined,
  MenuUnfoldOutlined,
  MenuFoldOutlined,
} from '@ant-design/icons-vue';

const selectedKeys = ref<string[]>(['1']);
const collapsed = ref<boolean>(false);
const router = useRouter();

onMounted(() => {
  // 监听路由变化
  console.log('onMounted',router);
  
});

</script>
