import request from '@/utils/request';

export function getArticleList(params: any) {
    return request({
        url: '/getSonChannelContent',
        method: 'get',
        params
    });
}