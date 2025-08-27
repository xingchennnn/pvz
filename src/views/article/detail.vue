<template>
  <a-modal
    v-model:open="modal2Visible"
    :title="modelTitle"
    centered
    keyboard
    destroyOnClose
    @ok="handleOk"
    width="30%"
    okText="打印"
    :confirm-loading="printLoading"
  >
    <div id="art_detail">
      <div class="p-3 w-full h-full overflow-y-auto [&_img]:w-[80%] art-detail">
        <div class="text-2xl font-bold mb-3 text-align-center">
          {{ detailData.title }}
        </div>
        <hr class="my-3" />
        <div
          style="
            display: flex;
            align-items: center;
            justify-content: space-between;
            margin-bottom: 0.75rem;
            color: #999;
            font-size: 12px;
          "
        >
          <div class="text-lg c-[var(--text-color)] font-size-12px">
            发表时间：{{ detailData.createTime }}
          </div>
          <div class="text-lg c-[var(--text-color)] font-size-12px">
            来源：{{ detailData.dataSource }}
          </div>
          <div class="text-lg c-[var(--text-color)] font-size-12px">
            浏览次数：{{ detailData.clickNum }}
          </div>
        </div>

        <div v-html="articleDetails" class="h-50vh overflow-y-auto"></div>
      </div>
    </div>
  </a-modal>
</template>

<script lang="ts" setup>
import { getArticleDetail } from "@/api/article";
import { onMounted, ref } from "vue";
import { useRoute } from "vue-router";
// import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

const route = useRoute();
const articleId = route.query.id as string;

const articleDetails = ref("");
const detailData = ref<any>({});

const modal2Visible = ref(false); // 弹窗状态
const modelTitle = ref("文章详情"); // 弹窗标题
const printLoading = ref(false); // 打印状态

// 获取文章详情
function getDetail(articleId: string) {
  if (!articleId) return;
  // TODO: 获取文章详情
  getArticleDetail({ contentId: articleId }).then((res: any) => {
    // getCurrentWebviewWindow().setTitle(res.data.title); // 设置标题
    // modelTitle.value = res.data.title;
    detailData.value = res.data;
    articleDetails.value = replaceImgUrl(res.data.content);
  });
}

// 图片替换
const replaceImgUrl = (content: string) => {
  const imgReg = /<img.*?src=[\'"](.*?)[\'"].*?>/g;

  const newContent = content.replace(imgReg, (_match, _src) => {
    // return `<img style="width:80%;" src="https://www.cqgh.org/guns/sysFileInfo/private/huaweiPreview?fileLocation=7&filePath=wangzhan${src}" alt="" />`;
    return '';
  });

  return newContent;
};

const openModal = (articleId: string) => {
  getDetail(articleId);
  modal2Visible.value = true;
};

function handleOk() {
  // TODO: 打印文章详情
  console.log("ok art_detail");
  printLoading.value = true;
  doPrint3("art_detail");
}

function doPrint3(idName: string) {
  //判断iframe是否存在，不存在则创建iframe
  let iframe: HTMLIFrameElement | null = document.getElementById(
    "print-iframe"
  ) as HTMLIFrameElement | null;
  // 卸载iframe
  //把iframe指向空白页面，这样可以释放大部分内存。
  iframe && (iframe.src = "about:blank");
  try {
    iframe?.contentWindow?.document.write("");
    iframe?.contentWindow?.document.clear();
  } catch (e) {}
  //把iframe从页面移除
  iframe?.parentNode?.removeChild(iframe);
  iframe = null;

  if (!iframe) {
    let el: Element = document.getElementById(idName) as Element;
    iframe = document.createElement("IFRAME") as HTMLIFrameElement;
    let doc: Document | null = null;
    iframe.setAttribute("id", "print-iframe");
    iframe.setAttribute(
      "style",
      "position:absolute;width:0px;height:0px;left:-500px;top:-500px;"
    );
    document.body.appendChild(iframe);
    doc = iframe!.contentWindow!.document;
    //这里可以自定义样式
    //doc.write("<LINK rel="stylesheet" type="text/css" href="css/print.css">");
    doc?.write("<div>" + el.innerHTML + "</div>");
    doc?.close();
    iframe?.contentWindow?.focus();
  }
  setTimeout(() => {
    //调用iframe的打印功能
    iframe?.contentWindow?.print();
    if (navigator.userAgent.indexOf("MSIE") > 0) {
      document.body.removeChild(iframe);
    }
    printLoading.value = false;
  }, 1000);
}

onMounted(() => {
  getDetail(articleId);
});

defineExpose({
  openModal,
});
</script>

<style scoped>
.art-detail {
  --text-color: #999;
}
</style>
