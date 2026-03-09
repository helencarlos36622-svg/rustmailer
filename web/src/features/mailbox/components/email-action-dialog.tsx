/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { useMemo } from 'react';
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Button } from '@/components/ui/button';
import { useMutation } from '@tanstack/react-query';
import { toast } from '@/hooks/use-toast';
import { Addr, EmailEnvelope, formatAddressList } from '../data/schema';
import { forward_mail, reply_mail } from '@/api/mailbox/envelope/api';
import useMinimalAccountList from '@/hooks/use-minimal-account-list';
import { z } from 'zod';
import { useForm, useFieldArray } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import {
    Form,
    FormControl,
    FormField,
    FormItem,
    FormLabel,
    FormMessage,
} from '@/components/ui/form';
import { Input } from '@/components/ui/input';
import { Textarea } from '@/components/ui/textarea';

// Define base schema with all fields but dynamic validation
const createEmailSchema = (action: EmailAction) => z.object({
    content: z.string().min(1, 'Message content is required'),
    toEmails: z.array(
        z.object({
            address: z.string().email('Invalid email address').min(1, 'Email is required'),
        })
    ).optional().superRefine((val, ctx) => {
        // Only validate toEmails for forward action
        if (action === 'forward' && (!val || val.length === 0)) {
            ctx.addIssue({
                code: z.ZodIssueCode.custom,
                message: 'At least one recipient is required',
            });
        }
    }),
    ccEmails: z.array(
        z.object({
            address: z.string().email('Invalid email address').min(1, 'Email is required'),
        })
    ).optional(),
});

type EmailFormData = z.infer<ReturnType<typeof createEmailSchema>>;

export type EmailAction = 'reply' | 'replyAll' | 'forward';

interface EmailActionDialogProps {
    open: boolean;
    action: EmailAction;
    onOpenChange: (open: boolean) => void;
    currentEnvelope?: EmailEnvelope | undefined;
    currentAccountId?: number | undefined;
}

function mergeAndFormatAddresses(envelope: EmailEnvelope, email: string): string {
    const addresses: Addr[] = [];
    if (envelope.from) {
        addresses.push(envelope.from);
    }

    if (envelope.to?.length) {
        const filteredTo = envelope.to.filter(addr => addr.address !== email);
        addresses.push(...filteredTo);
    }

    return formatAddressList(addresses).join(", ");
}

const filterOutEmail = (addresses: Addr[], emailToExclude: string): Addr[] => {
    return addresses.filter(addr => addr.address !== emailToExclude);
};

function formatFromAddress(envelope: EmailEnvelope): string {
    const addresses: Addr[] = [];
    if (envelope.from) {
        addresses.push(envelope.from);
    }
    return formatAddressList(addresses).join(", ");
}

export function EmailActionDialog({
    open,
    action,
    onOpenChange,
    currentEnvelope,
    currentAccountId
}: EmailActionDialogProps) {
    const { getEmailById } = useMinimalAccountList();
    const email = getEmailById(currentAccountId!);

    const schema = useMemo(() => createEmailSchema(action), [action]);

    const form = useForm<EmailFormData>({
        resolver: zodResolver(schema),
        defaultValues: {
            content: '',
            toEmails: action === 'forward' ? [{ address: '' }] : [],
            ccEmails: [],
        },
    });

    const { fields: toFields, append: appendTo, remove: removeTo } = useFieldArray({
        control: form.control,
        name: 'toEmails',
    });

    const { fields: ccFields, append: appendCc, remove: removeCc } = useFieldArray({
        control: form.control,
        name: 'ccEmails',
    });

    const { mutate, isPending } = useMutation({
        mutationFn: (data: EmailFormData) => {
            if (action === "reply" || action === "replyAll") {
                return reply_mail(currentAccountId!, {
                    mailbox_name: currentEnvelope?.mailbox_name,
                    id: currentEnvelope?.id,
                    text: data.content,
                    reply_all: action === "replyAll",
                    include_original: true,
                    include_all_attachments: false,
                    send_control: {
                        save_to_sent: false,
                        dry_run: false,
                        enable_tracking: false
                    }
                });
            } else {
                return forward_mail(currentAccountId!, {
                    mailbox_name: currentEnvelope?.mailbox_name,
                    id: currentEnvelope?.id,
                    text: data.content,
                    to: data.toEmails?.map(email => ({ address: email.address, name: null })),
                    cc: data.ccEmails?.length ? data.ccEmails.map(email => ({ address: email.address, name: null })) : null,
                    include_original: true,
                    include_all_attachments: false,
                    send_control: {
                        save_to_sent: false,
                        dry_run: false,
                        enable_tracking: false
                    }
                });
            }
        },
        onSuccess: () => {
            onOpenChange(false);
            form.reset();
            toast({
                title: 'Success',
                description: `Email ${action} task submitted successfully.`,
            });
        },
        onError: (error: Error) => {
            toast({
                title: 'Failed to send',
                description: error.message,
                variant: 'destructive',
            });
        },
    });

    const { title, recipients } = useMemo(() => {
        if (!currentEnvelope) return { title: '', recipients: null };

        switch (action) {
            case 'reply':
                return {
                    title: `Reply to ${formatFromAddress(currentEnvelope)}`,
                    recipients: `To: ${formatFromAddress(currentEnvelope)}`,
                };
            case 'replyAll':
                return {
                    title: `Reply all`,
                    recipients: (
                        <>
                            <div>To: {mergeAndFormatAddresses(currentEnvelope, email!)}</div>
                            {currentEnvelope.cc && currentEnvelope.cc.length > 0 && (
                                <div>
                                    Cc: {formatAddressList(
                                        filterOutEmail(currentEnvelope.cc, email!)
                                    )}
                                </div>
                            )}
                        </>
                    ),
                };
            case 'forward':
                return {
                    title: `Forward message`,
                    recipients: null,
                };
            default:
                return { title: '', recipients: null };
        }
    }, [action, currentEnvelope, email]);

    const onSubmit = (data: EmailFormData) => {
        mutate(data);
    };

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="w-full md:max-w-3xl h-[60vh] flex flex-col">
                <DialogHeader className="text-left">
                    <DialogTitle>{title}</DialogTitle>
                    <DialogDescription>
                        {action !== 'forward' ? 'Re: ' : 'Fwd: '}
                        {currentEnvelope?.subject}
                    </DialogDescription>
                </DialogHeader>

                <Form {...form}>
                    <form onSubmit={form.handleSubmit(onSubmit)} className="flex-1 flex flex-col">
                        {action !== 'forward' && recipients && (
                            <div className="my-2 text-sm space-y-1">{recipients}</div>
                        )}

                        <ScrollArea className="w-full max-h-[40vh] rounded-md border flex-1">
                            <div className="p-4 space-y-4">
                                {action === 'forward' && (
                                    <>
                                        {/* To Emails */}
                                        <div className="space-y-2">
                                            <FormLabel>To</FormLabel>
                                            {toFields.map((field, index) => (
                                                <FormField
                                                    key={field.id}
                                                    control={form.control}
                                                    name={`toEmails.${index}.address`}
                                                    render={({ field }) => (
                                                        <FormItem className="flex items-center gap-2">
                                                            <FormControl>
                                                                <Input
                                                                    placeholder="Enter email address"
                                                                    {...field}
                                                                />
                                                            </FormControl>
                                                            {toFields.length > 1 && (
                                                                <Button
                                                                    type="button"
                                                                    variant="destructive"
                                                                    size="sm"
                                                                    onClick={() => removeTo(index)}
                                                                >
                                                                    Remove
                                                                </Button>
                                                            )}
                                                            <FormMessage />
                                                        </FormItem>
                                                    )}
                                                />
                                            ))}
                                            <Button
                                                type="button"
                                                variant="outline"
                                                size="sm"
                                                onClick={() => appendTo({ address: '' })}
                                            >
                                                Add To Email
                                            </Button>
                                        </div>

                                        {/* CC Emails */}
                                        <div className="space-y-2">
                                            <FormLabel>Cc</FormLabel>
                                            {ccFields.map((field, index) => (
                                                <FormField
                                                    key={field.id}
                                                    control={form.control}
                                                    name={`ccEmails.${index}.address`}
                                                    render={({ field }) => (
                                                        <FormItem className="flex items-center gap-2">
                                                            <FormControl>
                                                                <Input
                                                                    placeholder="Enter CC email address"
                                                                    {...field}
                                                                />
                                                            </FormControl>
                                                            <Button
                                                                type="button"
                                                                variant="destructive"
                                                                size="sm"
                                                                onClick={() => removeCc(index)}
                                                            >
                                                                Remove
                                                            </Button>
                                                            <FormMessage />
                                                        </FormItem>
                                                    )}
                                                />
                                            ))}
                                            <Button
                                                type="button"
                                                variant="outline"
                                                size="sm"
                                                onClick={() => appendCc({ address: '' })}
                                                className={ccFields.length === 0 ? "ml-4" : ""}
                                            >
                                                Add CC Email
                                            </Button>
                                        </div>
                                    </>
                                )}

                                <FormField
                                    control={form.control}
                                    name="content"
                                    render={({ field }) => (
                                        <FormItem className="flex-1">
                                            <FormControl>
                                                <Textarea
                                                    className="w-full h-full min-h-[300px] p-4 outline-none resize-none"
                                                    placeholder={
                                                        action === 'forward'
                                                            ? 'Add a message to forward with...'
                                                            : 'Type your reply here...'
                                                    }
                                                    {...field}
                                                    autoFocus
                                                />
                                            </FormControl>
                                            <FormMessage />
                                        </FormItem>
                                    )}
                                />
                            </div>
                        </ScrollArea>

                        <div className="flex justify-end gap-2 pt-4">
                            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
                                Cancel
                            </Button>
                            <Button type="submit" disabled={isPending}>
                                {isPending ? (
                                    <span className="flex items-center gap-2">
                                        <Spinner /> Sending...
                                    </span>
                                ) : (
                                    action === 'forward' ? 'Forward' : 'Send'
                                )}
                            </Button>
                        </div>
                    </form>
                </Form>
            </DialogContent>
        </Dialog>
    );
}

// Loading spinner component
function Spinner() {
    return (
        <span className="inline-block h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent" />
    );
}