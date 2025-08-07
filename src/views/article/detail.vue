<template>
    <div class="p-3 w-full h-full overflow-y-auto [&_img]:w-[80%] art-detail">
        <div class="text-2xl font-bold mb-3 text-align-center">{{ detailData.title }}</div>
        <hr class="my-3" />
        <div class="flex justify-between items-center mb-3 ">
            <div class="text-lg c-[var(--text-color)]">发表时间：{{ detailData.createTime }}</div>
            <div class="text-lg c-[var(--text-color)]">来源：{{ detailData.dataSource }}</div>
            <div class="text-lg c-[var(--text-color)]">浏览次数：{{ detailData.clickNum }}</div>
        </div>

        <div v-html="articleDetails"></div>
    </div>
</template>

<script lang="ts" setup>
import { getArticleDetail } from '@/api/article'
import { onMounted, ref } from 'vue'
import { useRoute } from 'vue-router'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'

const route = useRoute()
const articleId = route.query.id as string

const articleDetails = ref('')
const detailData = ref<any>({})

function getDetail() {
    // TODO: 获取文章详情
    getArticleDetail({ contentId: articleId }).then(res => {
        getCurrentWebviewWindow().setTitle(res.data.title)
        detailData.value = res.data
        articleDetails.value = replaceImgUrl(res.data.content)
    })
}

// 图片替换
const replaceImgUrl = (content: string) => {
    const imgReg = /<img.*?src=[\'"](.*?)[\'"].*?>/g;

    const newContent = content.replace(imgReg, (_match, src) => {
        return `<img src="https://www.cqgh.org/guns/sysFileInfo/private/huaweiPreview?fileLocation=7&filePath=wangzhan${src}" alt="" />`;
    });

    return newContent;
}








onMounted(() => {
    getDetail()
})
</script>

<style scoped>
.art-detail {
    --text-color: #999;
}
</style>