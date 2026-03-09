/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */


import useMinimalAccountList from "@/hooks/use-minimal-account-list";
import { VirtualizedSelect } from "@/components/virtualized-select";
import { Button } from "@/components/ui/button";
import { useNavigate } from "@tanstack/react-router";


interface AccountSwitcherProps {
    onAccountSelect: (accountId: number) => void,
    defaultAccountId?: number,
}

export function AccountSwitcher({
    onAccountSelect,
    defaultAccountId
}: AccountSwitcherProps) {
    const { accountsOptions, isLoading } = useMinimalAccountList();
    const navigate = useNavigate()

    if (isLoading) {
        return <div>Loading...</div>;
    }

    return (
        <VirtualizedSelect
            className='w-full mr-8'
            isLoading={isLoading}
            options={accountsOptions}
            defaultValue={`${defaultAccountId}`}
            onSelectOption={(values) => onAccountSelect(parseInt(values[0], 10))}
            placeholder="Select an account"
            noItemsComponent={<div className='space-y-2'>
                <p>No active email account.</p>
                <Button variant={'outline'} className="py-1 px-3 text-xs" onClick={() => navigate({ to: '/accounts' })}>Add Email Account</Button>
            </div>}
        />
    );
}