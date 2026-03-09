/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import {
    FormField,
    FormItem,
    FormLabel,
    FormMessage,
    FormControl,
    FormDescription,
} from "@/components/ui/form";
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from "@/components/ui/select";
import { Input } from "@/components/ui/input";
import { useFormContext, useWatch } from "react-hook-form";
import { Account } from "./action-dialog";
import { PasswordInput } from "@/components/password-input";
import useProxyList from "@/hooks/use-proxy";

interface StepProps {
    isEdit: boolean;
}

export default function Step3({ isEdit }: StepProps) {
    const { control } = useFormContext<Account>();
    const { proxyOptions } = useProxyList();
    const smtpAuthMethod = useWatch({
        control,
        name: "smtp.auth.auth_type",
    });

    return (
        <>
            <div className="space-y-8">
                <FormField
                    control={control}
                    name="smtp.host"
                    render={({ field }) => (
                        <FormItem>
                            <FormLabel className="flex items-center justify-between">
                                SMTP Host:
                            </FormLabel>
                            <FormControl>
                                <Input placeholder="e.g smtp.example.com" {...field} />
                            </FormControl>
                            <FormMessage />
                        </FormItem>
                    )}
                />
                <FormField
                    control={control}
                    name="smtp.port"
                    render={({ field }) => (
                        <FormItem>
                            <FormLabel className="flex items-center justify-between">
                                SMTP Port:
                            </FormLabel>
                            <FormControl>
                                <Input type="number" placeholder="e.g 465" {...field} onChange={(e) => field.onChange(parseInt(e.target.value, 10))} />
                            </FormControl>
                            <FormMessage />
                        </FormItem>
                    )}
                />
                <FormField
                    control={control}
                    name="smtp.encryption"
                    render={({ field }) => (
                        <FormItem>
                            <FormLabel>IMAP Auth Method:</FormLabel>
                            <Select onValueChange={field.onChange} defaultValue={field.value}>
                                <FormControl>
                                    <SelectTrigger>
                                        <SelectValue placeholder="Select an authentication method" />
                                    </SelectTrigger>
                                </FormControl>
                                <SelectContent>
                                    <SelectItem value="Ssl">Ssl</SelectItem>
                                    <SelectItem value="StartTls">StartTls</SelectItem>
                                    <SelectItem value="None">None</SelectItem>
                                </SelectContent>
                            </Select>
                            <FormDescription>
                                Choose the authentication method for SMTP.
                            </FormDescription>
                            <FormMessage />
                        </FormItem>
                    )}
                />
                <FormField
                    control={control}
                    name="smtp.auth.auth_type"
                    render={({ field }) => (
                        <FormItem>
                            <FormLabel>SMTP Auth Method:</FormLabel>
                            <Select onValueChange={field.onChange} defaultValue={field.value}>
                                <FormControl>
                                    <SelectTrigger>
                                        <SelectValue placeholder="Select an authentication method" />
                                    </SelectTrigger>
                                </FormControl>
                                <SelectContent>
                                    <SelectItem value="OAuth2">OAuth2</SelectItem>
                                    <SelectItem value="Password">Password</SelectItem>
                                </SelectContent>
                            </Select>
                            <FormDescription>
                                Choose the authentication method for SMTP.
                            </FormDescription>
                            <FormMessage />
                        </FormItem>
                    )}
                />
                {smtpAuthMethod === "Password" && (
                    <FormField
                        control={control}
                        name="smtp.auth.password"
                        render={({ field }) => (
                            <FormItem>
                                <FormLabel className="flex items-center justify-between">
                                    SMTP Password:
                                </FormLabel>
                                <FormControl>
                                    <PasswordInput placeholder={isEdit ? "Leave empty to keep current password" : "Enter your password"} {...field} />
                                </FormControl>
                                <FormMessage />
                                {isEdit && (
                                    <FormDescription>
                                        Leave empty to keep the existing password, or enter a new password to update it.
                                    </FormDescription>
                                )}
                            </FormItem>
                        )}
                    />
                )}
                <FormField
                    control={control}
                    name='smtp.use_proxy'
                    render={({ field }) => (
                        <FormItem>
                            <FormLabel className="flex items-center justify-between">Use Proxy(optional):</FormLabel>
                            <FormControl>
                                <Select
                                    onValueChange={(val) => field.onChange(Number(val))}
                                    defaultValue={field.value?.toString()}
                                >
                                    <FormControl>
                                        <SelectTrigger>
                                            <SelectValue placeholder="Select a proxy" />
                                        </SelectTrigger>
                                    </FormControl>
                                    <SelectContent>
                                        {proxyOptions && proxyOptions.length > 0 ? (
                                            proxyOptions.map((option) => (
                                                <SelectItem key={option.value} value={option.value.toString()}>
                                                    {option.label}
                                                </SelectItem>
                                            ))
                                        ) : (
                                            <SelectItem disabled value="__none__">No proxy available</SelectItem>
                                        )}
                                    </SelectContent>
                                </Select>
                            </FormControl>
                            <FormDescription className='flex-1'>
                                Use a SOCKS5 proxy for SMTP connections.
                            </FormDescription>
                            <FormMessage />
                        </FormItem>
                    )}
                />
            </div>
        </>
    );
}