<template>
  <main class="container">
    <h1 :class="{ flicker: offline  }">植物大战僵尸杂交版V3.9</h1>
    <a-flex direction="column" justify="center" align="center" style="margin: 20px 0">
      <a-card title="基本信息" :bordered="false" style="width: 300px">
        <div class="col-md-6">
          进程id: {{ pid }}
        </div>
        <div class="col-md-6">
          阳光值: {{ sunValue }}
        </div>
      </a-card>
    </a-flex>



    <form class="row" @submit.prevent="greet">
      <input id="greet-input" v-model="name" placeholder="Enter a name..." />
      <button type="submit">Greet</button>
    </form>
    <p>{{ greetMsg }}</p>
  </main>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { invoke, } from "@tauri-apps/api/core";
import { callRustType } from "./types/callRust";
import { MusicPlayer } from "./utils";
import { message } from 'ant-design-vue';
// import { Flex , Card } from 'ant-design-vue';

const greetMsg = ref("");
const name = ref("");
const mp3 = new MusicPlayer('/music/ding.mp3');

const pid = ref(0);
const sunValue = ref(0);

const offline = ref(false);

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  // greetMsg.value = await invoke("greet", { name: name.value });
  if (name.value === '1') {
    greetMsg.value = await invoke("call_rust", { types: callRustType.AddSun, text: "添加阳光" });
    message.success(greetMsg.value);
  } else if (name.value === '2') {
    const JSONdata: string = await invoke("call_rust", { types: callRustType.GetSun, text: "请求阳光值" });
    const data = JSON.parse(JSONdata);
    // console.log(data);
    sunValue.value = data.sun_value;
    pid.value = data.pid;
  } else if (name.value === '3') {
    greetMsg.value = await invoke("call_rust", { types: callRustType.COOLING, text: "修改冷却" });
  }
  mp3.setCurrentTime(0)
  mp3.play()
}

onMounted(() => {
  const intervalId = setInterval(() => {
    getSunValue()
  }, 2000)
  onUnmounted(() => {
    clearInterval(intervalId)
    mp3.stop()
  })
})

async function getSunValue() {
  const JSONdata: string = await invoke("call_rust", { types: callRustType.GetSun, text: "请求阳光值" });
  if (JSONdata.startsWith("错误")) {
    // message.error("获取阳光值失败，没有开始游戏");
    // console.log("获取阳光值失败，没有开始游戏" ,  new RegExp(/错误/g).test(JSONdata) );
    offline.value = true;
    sunValue.value = 0;
    pid.value = 0;
    return;
  } else {
    offline.value = false;
  }
  console.log(JSONdata);
  const data = JSON.parse(JSONdata);
  sunValue.value = data.sun_value;
  pid.value = data.pid;
  if (data.sun_value < 5000) {
    // console.log("阳光值小于1000，+2000");
    await invoke("call_rust", { types: callRustType.AddSun, text: "阳光值小于5000，+5000" });
  }
}


</script>

<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color: #0f0f0f;
  background-color: #ececec;


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