/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { list_proxy } from "@/api/system/api";
import { useQuery } from "@tanstack/react-query";

const useProxyList = () => {
    const { data: proxyList, ...rest } = useQuery({
        queryKey: ['proxy-list'],
        queryFn: list_proxy,
        staleTime: 10 * 60 * 1000
    });

    const proxyOptions = proxyList
        ? proxyList.map(proxy => ({
            label: proxy.url,
            value: `${proxy.id}`,
        }))
        : [];


    const getUrlById = (id: string | number) => {
        if (!proxyList) return null;
        const proxy = proxyList.find(a => `${a.id}` === `${id}`);
        return proxy?.url || null;
    };

    return {
        proxyOptions,
        proxyList,
        getUrlById,
        ...rest
    };
};

export default useProxyList;