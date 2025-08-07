<template>
    <div>
        <a-table :columns="columns" :data-source="articleList" bordered :loading="loading" :pagination="false"
            resizable row-class-name="row-class-name">
            <template #bodyCell="{ column, record }">
                <template v-if="column.key === 'action'">
                    <div class="flex justify-center items-center min-w-130px w-130px">
                        <a-button type="link" @click="handleView(record)">详情</a-button>
                        <a-button type="link" @click="handleWord(record)">word查看</a-button>
                    </div>
                </template>
            </template>
        </a-table>
        <div class="flex justify-end p-1">
            <a-pagination v-model:current="queryParam.pageNum" v-model:page-size="queryParam.pageSize" :total="total"
                :show-total="(total: number) => `共${total}条`" @change="getLists" />
        </div>
    </div>
</template>

<script setup lang="ts">
import { getArticleList } from '@/api/article';
import { message } from 'ant-design-vue';
import { onMounted, reactive, ref } from 'vue';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';

const articleList = ref(); // 文章列表

const loading = ref(false); // 加载状态
const total = ref(0); // 总页数
const queryParam = reactive({
    pageNum: 1,
    pageSize: 10,
    siteId: 35,  // 站点ID  --死参数
    channelId: 2201,  // 频道ID  --死参数
})


const getLists = () => {
    // 这里是请求文章列表的逻辑
    loading.value = true;
    getArticleList(queryParam).then((res: any) => {
        articleList.value = res.rows;
        total.value = res.total;
    }).catch(err => {
        console.log(err);
        message.error(err.message);
    }).finally(() => {
        loading.value = false;
    });
}


const columns = [
    {
        title: '文章名字',
        dataIndex: 'title',
        key: 'title',
        width: '60%',
        ellipsis: true,
    },
    {
        title: '发布时间',
        dataIndex: 'releaseTime',
        key: 'releaseTime',
        align: 'center',
        width: '200px',
        ellipsis: true,
    },
    {
        title: '操作',
        key: 'action',
        width: '20%',
        ellipsis: true,
        fixed: 'right',
    },
];


const handleView = (record: any) => {
    // 这里是详情页的逻辑
    console.log(record);
    const webviewWindow1 = new WebviewWindow("articleDetail", {
        title: '文章详情',
        url: '/#/article/detail?id=' + record.contentId,
        width: 800,
        height: 800,
        center: true,
    });
    webviewWindow1.once('tauri://created', function () {
        // webview successfully created
        console.log('webview created');

    });
    webviewWindow1.once('tauri://error', function (e) {
        // an error happened creating the webview
        console.log('webview error', e);
    });
}

const handleWord = (record: any) => {
    // 这里是word查看的逻辑
    console.log(record);
}


onMounted(() => {
    getLists();
})


</script>

<style scoped></style>