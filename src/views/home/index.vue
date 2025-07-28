<template>
  <main class="container">
    <!-- <h1 :class="{ flicker: offline  }">植物大战僵尸杂交版V3.9</h1> -->
    <a-flex
      direction="column"
      justify="center"
      align="center"
      class="mt-20px mb-20px"
    >
      <a-card title="基本信息" :bordered="false" style="width: 300px">
        <div class="col-md-6">进程id: {{ pid }}</div>
        <div class="col-md-6">阳光值: {{ sunValue }}</div>
      </a-card>
    </a-flex>

    <!-- <form class="row" @submit.prevent="greet">
      <a-input id="greet-input" v-model="name" placeholder="Enter a name..." />
      <a-button type="submit">Greet</a-button>
    </form> -->
    <div class="flex justify-center items-center w-full">
      <div class="w-40% flex gap-2">
        <a-input v-model:value="name" placeholder="请输入代码" />
        <a-button type="primary" @click="greet">调用</a-button>
      </div>
    </div>
    <p>{{ greetMsg }}</p>
  </main>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { callRustType } from "@/types/callRust";
import { MusicPlayer } from "@/utils";
import { message } from "ant-design-vue";
import { callRust } from "@/utils/callRust";

const greetMsg = ref("");
const name = ref("");
const mp3 = new MusicPlayer("/music/ding.mp3");

const pid = ref(0);
const sunValue = ref(0);

const offline = ref(false);

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  // greetMsg.value = await invoke("greet", { name: name.value });

  switch (name.value) {
    case "1":
      callRust(callRustType.AddSun, "添加阳光").then((res) => {
        greetMsg.value = res;
      });
      break;
    case "2":
      callRust(callRustType.GetSun, "请求阳光值").then((res) => {
        if (res.startsWith("错误")) {
          message.error("获取阳光值失败，没有开始游戏");
          sunValue.value = 0;
          pid.value = 0;
          message.error(res);
          return;
        }
        const data = JSON.parse(res);
        sunValue.value = data.sun_value;
        pid.value = data.pid;
      });
      break;
    case "3":
      callRust(callRustType.COOLING, "修改冷却").then((res) => {
        greetMsg.value = res;
      });
      break;
    default:
      callRust(callRustType.NONE, name.value).then((res) => {
        greetMsg.value = res;
      });
      break;
  }
  mp3.setCurrentTime(0);
  mp3.play();
}

onMounted(() => {
  const intervalId = setInterval(() => {
    getSunValue();
  }, 2000);
  onUnmounted(() => {
    clearInterval(intervalId);
    mp3.stop();
  });
});

function getSunValue() {
  callRust(callRustType.GetSun, "请求阳光值")
    .then((JSONdata) => {
      offline.value = false;
      const data = JSON.parse(JSONdata);
      sunValue.value = data.sun_value;
      pid.value = data.pid;
      if (data.sun_value < 5000) {
        callRust(callRustType.AddSun, "阳光值小于5000，+5000");
      }
    })
    .catch(() => {
      offline.value = true;
      sunValue.value = 0;
      pid.value = 0;
    });
}
</script>

<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color: #0f0f0f;
  /* background-color: #ececec; */

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

.container {
  margin: 0;
  padding-top: 10vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  text-align: center;
}

input,
button {
  border-radius: 8px;
  border: 1px solid transparent;
  padding: 0.6em 1.2em;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  color: #0f0f0f;
  background-color: #ffffff;
  transition: border-color 0.25s;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
}

button {
  cursor: pointer;
}

button:hover {
  border-color: #396cd8;
}

button:active {
  border-color: #396cd8;
  background-color: #e8e8e8;
}

input,
button {
  outline: none;
}

#greet-input {
  margin-right: 5px;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }

  a:hover {
    color: #24c8db;
  }

  input,
  button {
    color: #ffffff;
    background-color: #0f0f0f98;
  }

  button:active {
    background-color: #0f0f0f69;
  }
}

.flicker {
  animation: flickerS 2s linear infinite;
  color: red;
}

@keyframes flickerS {
  0% {
    opacity: 1;
  }

  50% {
    opacity: 0.3;
  }

  100% {
    opacity: 1;
  }
}
</style>
