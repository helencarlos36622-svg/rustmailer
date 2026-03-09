/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { useFormContext } from "react-hook-form";
import { Account } from "./action-dialog";
import { Accordion, AccordionItem, AccordionTrigger, AccordionContent } from "@/components/ui/accordion";

export default function Step4() {
    const { getValues } = useFormContext<Account>();
    const summaryData = getValues();

    return (
        <>
            <div className="p-5 rounded-xl">
                <Accordion type="multiple" defaultValue={['email', 'name', 'minimal_sync', 'isolated_index', 'imap', 'smtp', 'date_since', 'folder_limit', 'sync_folders', 'language', 'sync_interval']}>
                    <AccordionItem key="email" value="email">
                        <AccordionTrigger className="font-medium capitalize text-gray-600">
                            Email:
                        </AccordionTrigger>
                        <AccordionContent>
                            {summaryData.email}
                        </AccordionContent>
                    </AccordionItem>
                    <AccordionItem key="name" value="name">
                        <AccordionTrigger className="font-medium capitalize text-gray-600">
                            Name:
                        </AccordionTrigger>
                        <AccordionContent>
                            {summaryData.name ?? "n/a"}
                        </AccordionContent>
                    </AccordionItem>
                    <AccordionItem key="minimal-sync" value="minimal_sync">
                        <AccordionTrigger className="font-medium capitalize text-gray-600">
                            Minimal Sync:
                        </AccordionTrigger>
                        <AccordionContent>
                            {`${summaryData.minimal_sync}`}
                        </AccordionContent>
                    </AccordionItem>
                    <AccordionItem key="imap" value='imap'>
                        <AccordionTrigger className="font-medium capitalize text-gray-600">
                            Imap:
                        </AccordionTrigger>
                        <AccordionContent>
                            <div className="overflow-x-auto">
                                <table className="min-w-full divide-y">
                                    <tbody className="divide-y">
                                        <tr>
                                            <td className="px-6 py-2 whitespace-nowrap text-sm font-medium text-gray-600">host:</td>
                                            <td className="px-6 py-2 whitespace-nowrap text-sm">{summaryData.imap.host}</td>
                                        </tr>
                                        <tr>
                                            <td className="px-6 py-2 whitespace-nowrap text-sm font-medium text-gray-600">port:</td>
                                            <td className="px-6 py-2 whitespace-nowrap text-sm">{summaryData.imap.port}</td>
                                        </tr>
                                        <tr>
                                            <td className="px-6 py-2 whitespace-nowrap text-sm font-medium text-gray-600">encryption:</td>
                                            <td className="px-6 py-2 whitespace-nowrap text-sm">{summaryData.imap.encryption}</td>
                                        </tr>
                                        <tr>
                                            <td className="px-6 py-2 whitespace-nowrap text-sm font-medium text-gray-600">auth_type:</td>
                                            <td className="px-6 py-2 whitespace-nowrap text-sm">{summaryData.imap.auth.auth_type}</td>
                                        </tr>
                                        {summaryData.imap.auth.auth_type === 'Password' && (
                                            <tr>
                                                <td className="px-6 py-2 whitespace-nowrap text-sm font-medium text-gray-600">password:</td>
                                                <td className="px-6 py-2 whitespace-nowrap text-sm break-words">
                                                    {summaryData.imap.auth.password}
                                                </td>
                                            </tr>
                                        )}
                                        <tr>
                                            <td className="px-6 py-2 whitespace-nowrap text-sm font-medium text-gray-600">use proxy:</td>
                                            <td className="px-6 py-2 whitespace-nowrap text-sm">{summaryData.imap.use_proxy ? "true" : "false"}</td>
                                        </tr>
                                    </tbody>
                                </table>
                            </div>
                        </AccordionContent>
                    </AccordionItem>
                    <AccordionItem key="smtp" value='smtp'>
                        <AccordionTrigger className="font-medium capitalize text-gray-600">
                            Smtp
                        </AccordionTrigger>
                        <AccordionContent>
                            <div className="overflow-x-auto">
                                <table className="min-w-full divide-y">
                                    <tbody className="divide-y">
                                        <tr>
                                            <td className="px-6 py-2 whitespace-nowrap text-sm font-medium text-gray-600">host:</td>
                                            <td className="px-6 py-2 whitespace-nowrap text-sm">{summaryData.smtp.host}</td>
                                        </tr>
                                        <tr>
                                            <td className="px-6 py-2 whitespace-nowrap text-sm font-medium text-gray-600">port:</td>
                                            <td className="px-6 py-2 whitespace-nowrap text-sm">{summaryData.smtp.port}</td>
                                        </tr>
                                        <tr>
                                            <td className="px-6 py-2 whitespace-nowrap text-sm font-medium text-gray-600">encryption:</td>
                                            <td className="px-6 py-2 whitespace-nowrap text-sm">{summaryData.smtp.encryption}</td>
                                        </tr>
                                        <tr>
                                            <td className="px-6 py-2 whitespace-nowrap text-sm font-medium text-gray-600">auth_type:</td>
                                            <td className="px-6 py-2 whitespace-nowrap text-sm">{summaryData.smtp.auth.auth_type}</td>
                                        </tr>
                                        {summaryData.smtp.auth.auth_type === 'Password' && (
                                            <tr>
                                                <td className="px-6 py-2 whitespace-nowrap text-sm font-medium text-gray-600">password:</td>
                                                <td className="px-6 py-2 whitespace-nowrap text-sm break-words">
                                                    {summaryData.smtp.auth.password}
                                                </td>
                                            </tr>
                                        )}
                                        <tr>
                                            <td className="px-6 py-2 whitespace-nowrap text-sm font-medium text-gray-600">use proxy:</td>
                                            <td className="px-6 py-2 whitespace-nowrap text-sm">{summaryData.smtp.use_proxy ? "true" : "false"}</td>
                                        </tr>
                                    </tbody>
                                </table>
                            </div>
                        </AccordionContent>
                    </AccordionItem>
                    <AccordionItem key="date_since" value='date_since'>
                        <AccordionTrigger className="font-medium capitalize text-gray-600">
                            Date Selection:
                        </AccordionTrigger>
                        <AccordionContent>
                            {summaryData.date_since?.fixed ?
                                'since ' + summaryData.date_since.fixed
                                : summaryData.date_since?.relative && summaryData.date_since.relative.value && summaryData.date_since.relative.unit ?
                                    'recent ' + summaryData.date_since.relative.value + ' ' + summaryData.date_since.relative.unit
                                    : 'n/a'}
                        </AccordionContent>
                    </AccordionItem>
                    <AccordionItem key="folder_limit" value='folder_limit'>
                        <AccordionTrigger className="font-medium capitalize text-gray-600">
                            Folder Sync Limit:
                        </AccordionTrigger>
                        <AccordionContent>
                            {summaryData.folder_limit ? summaryData.folder_limit : 'n/a'}
                        </AccordionContent>
                    </AccordionItem>
                    <AccordionItem key="sync_interval" value='sync_interval'>
                        <AccordionTrigger className="font-medium capitalize text-gray-600">
                            Sync Interval:
                        </AccordionTrigger>
                        <AccordionContent>
                            <div className="overflow-x-auto">
                                <table className="min-w-full divide-y">
                                    <tbody className="divide-y">
                                        <tr>
                                            <td className="px-6 py-2 whitespace-nowrap text-sm font-medium text-gray-600">Full Sync Interval:</td>
                                            <td className="px-6 py-2 whitespace-nowrap text-sm">{summaryData.full_sync_interval_min} min</td>
                                        </tr>
                                        <tr>
                                            <td className="px-6 py-2 whitespace-nowrap text-sm font-medium text-gray-600">Incremental Sync Interval:</td>
                                            <td className="px-6 py-2 whitespace-nowrap text-sm">{summaryData.incremental_sync_interval_sec} sec</td>
                                        </tr>
                                    </tbody>
                                </table>
                            </div>
                        </AccordionContent>
                    </AccordionItem>
                </Accordion>
            </div>
        </>
    );
}