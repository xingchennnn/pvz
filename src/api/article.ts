import request from '@/utils/request';

export function getArticleList(params: any) {
    return request({
        url: '/getSonChannelContent',
        method: 'get',
        params
    });
}

export function getArticleDetail(params: any) {
    return request({
        url: '/getInfo',
        method: 'get',
        params
    });
}