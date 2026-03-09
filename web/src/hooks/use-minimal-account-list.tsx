/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { minimal_account_list } from "@/api/account/api";
import { useQuery } from "@tanstack/react-query";

const useMinimalAccountList = () => {
  const { data: minimalList, ...rest } = useQuery({
    queryKey: ['minimal-account-list'],
    queryFn: minimal_account_list,
  });

  const accountsOptions = minimalList
    ? minimalList.map(account => ({
      label: account.email,
      value: `${account.id}`,
    }))
    : [];


  const getEmailById = (accountId: string | number) => {
    if (!minimalList) return null;
    const account = minimalList.find(a => `${a.id}` === `${accountId}`);
    return account?.email || null;
  };

  return {
    accountsOptions,
    minimalList,
    getEmailById,
    ...rest
  };
};

export default useMinimalAccountList;