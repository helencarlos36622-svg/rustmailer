/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { z } from 'zod'
import { useFieldArray, useForm } from 'react-hook-form'
import { Button } from '@/components/ui/button'
import {
    Form,
    FormControl,
    FormDescription,
    FormField,
    FormItem,
    FormLabel,
    FormMessage,
} from '@/components/ui/form'
import { Checkbox } from '@/components/ui/checkbox'
import { Textarea } from '@/components/ui/textarea'
import { Input } from '@/components/ui/input'
import { ScrollArea } from '@/components/ui/scroll-area'
import AceEditor from '@/components/ace-editor'
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from '@/components/ui/accordion'
import { Select } from '@/components/ui/select'
import { SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { MinusCircle, Plus } from 'lucide-react'
import { cn } from '@/lib/utils'
import { SelectDropdown } from '@/components/select-dropdown'
import { VirtualizedSelect } from '@/components/virtualized-select'
import { eventTypeSchema, eventTypeOptions } from './types'
import { Switch } from '@/components/ui/switch'
import { CodeEditorWithDraft } from '@/components/editor-with-draft'
import useProxyList from '@/hooks/use-proxy'

export const httpConfigSchema = z.object({
    target_url: z.string({
        required_error: 'Please provide a target URL.',
    }).url(),
    http_method: z.string().refine(val => ['Post', 'Put'].includes(val), {
        message: 'http_method must be either "Post" or "Put".',
    }).default('Post'),
    custom_headers: z.array(
        z.object({
            key: z.string({ required_error: 'Key is required' }).min(1, "Key cannot be empty"),
            value: z.string({ required_error: 'Value is required' }).min(1, "Value cannot be empty"),
        })
    ).optional(),
});

export const httpFormSchema = z.object({
    account_id: z.number().optional(),
    description: z.optional(
        z.string().max(255, { message: "Description must not exceed 255 characters." })
    ),
    enabled: z.boolean(),
    global: z.boolean(),
    http: httpConfigSchema,
    use_proxy: z.number().optional(),
    vrl_script: z.string().optional(),
    watched_events: z.array(eventTypeSchema).min(1, "At least one event type must be selected"),
}).superRefine((data, ctx) => {
    if (!data.global && !data.account_id) {
        ctx.addIssue({
            code: z.ZodIssueCode.custom,
            path: ['account_id'],
            message: "Please select an account when not using global mode",
        });
    }
});

export type HttpEventHookForm = z.infer<typeof httpFormSchema>

const httpMethodOptions = [
    { value: 'Post', label: 'Post' },
    { value: 'Put', label: 'Put' }
];

interface HttpFormProps {
    form: ReturnType<typeof useForm<HttpEventHookForm>>;
    accountsOptions: { label: string; value: string }[];
    isLoading: boolean;
    isUpdate: boolean;
    eventExamples?: Record<string, any>;
    inputJson: string | undefined;
    setInputJson: (value: string | undefined) => void;
    resolveResult: string | undefined;
    setResolveResult: (value: string | undefined) => void;
    runTest: () => void;
}

export function HttpForm({
    form,
    accountsOptions,
    isLoading,
    isUpdate,
    eventExamples,
    inputJson,
    setInputJson,
    resolveResult,
    setResolveResult,
    runTest
}: HttpFormProps) {
    const { fields, append, remove } = useFieldArray({
        name: 'http.custom_headers',
        control: form.control,
    });

    const isGlobal = form.watch('global');

    const { proxyOptions } = useProxyList();

    const handleGlobalToggle = (checked: boolean) => {
        form.setValue('global', checked);
        if (checked) {
            form.setValue('account_id', undefined);
        }
    };

    return (
        <Form {...form}>
            <form
                id='eventhook-form'
                className='space-y-5 flex-1'
            >
                {/* Global Switch */}
                <FormField
                    control={form.control}
                    name='global'
                    render={({ field }) => (
                        <FormItem className='flex flex-row items-center justify-between rounded-lg border p-4'>
                            <div className='space-y-0.5'>
                                <FormLabel className='text-base'>
                                    Global event hook
                                </FormLabel>
                                <FormDescription>
                                    A global event hook that handles events from all accounts, not just a specific one.
                                </FormDescription>
                            </div>
                            <FormControl>
                                <Switch
                                    checked={field.value}
                                    onCheckedChange={handleGlobalToggle}
                                    disabled={isUpdate} // Disable in edit mode
                                />
                            </FormControl>
                        </FormItem>
                    )}
                />

                {/* Account Selection */}
                <FormField
                    control={form.control}
                    name='account_id'
                    render={({ field }) => (
                        <FormItem className='space-y-1'>
                            <FormLabel className='mb-1'>Account:</FormLabel>
                            <FormControl>
                                <VirtualizedSelect
                                    options={accountsOptions}
                                    className='w-full'
                                    isLoading={isLoading}
                                    onSelectOption={(values) => field.onChange(parseInt(values[0], 10))}
                                    defaultValue={`${field.value}`}
                                    disabled={isUpdate || isGlobal}  // Disable if global or isUpdate
                                    placeholder="Select an account"
                                />
                            </FormControl>
                            <FormDescription>
                                {isGlobal
                                    ? "Global hooks apply to all accounts"
                                    : "Select the account associated with this event hook"}
                            </FormDescription>
                            <FormMessage />
                        </FormItem>
                    )}
                />

                {/* Enabled Checkbox */}
                <FormField
                    control={form.control}
                    name='enabled'
                    render={({ field }) => (
                        <FormItem className='flex flex-row items-center gap-x-2'>
                            <FormControl>
                                <Checkbox
                                    className='mt-2'
                                    checked={field.value}
                                    onCheckedChange={field.onChange}
                                />
                            </FormControl>
                            <FormLabel>Enabled</FormLabel>
                            <FormDescription>
                                Enable or disable this event hook.
                            </FormDescription>
                            <FormMessage />
                        </FormItem>
                    )}
                />

                {/* HTTP Configuration */}
                <div className="flex gap-4">
                    <FormField
                        control={form.control}
                        name='http.target_url'
                        render={({ field }) => (
                            <FormItem className='flex flex-col gap-y-1 space-y-0 w-1/2'>
                                <FormLabel className='mb-1'>Target URL:</FormLabel>
                                <FormControl>
                                    <Input
                                        placeholder='Enter the target URL for the HTTP request'
                                        {...field}
                                    />
                                </FormControl>
                                <FormDescription>
                                    The URL where the event data will be sent.
                                </FormDescription>
                                <FormMessage />
                            </FormItem>
                        )}
                    />
                    <FormField
                        control={form.control}
                        name='http.http_method'
                        render={({ field }) => (
                            <FormItem className='flex flex-col gap-y-1 space-y-0 w-1/2'>
                                <FormLabel className='mb-1'>HTTP Method:</FormLabel>
                                <FormControl>
                                    <SelectDropdown
                                        defaultValue={field.value}
                                        onValueChange={field.onChange}
                                        placeholder='Select the HTTP method'
                                        items={httpMethodOptions}
                                    />
                                </FormControl>
                                <FormMessage />
                            </FormItem>
                        )}
                    />
                </div>

                {/* Custom Headers */}
                <div>
                    {fields.map((_, index) => (
                        <div className="flex flex-col gap-4 sm:flex-row sm:items-center" key={index}>
                            <div className="flex flex-1 gap-4">
                                <FormField
                                    control={form.control}
                                    name={`http.custom_headers.${index}.key`}
                                    render={({ field }) => (
                                        <FormItem className="flex-1">
                                            <FormLabel className={cn(index !== 0 && "sr-only")}>Key:</FormLabel>
                                            <FormDescription className={cn(index !== 0 && "sr-only")}>
                                                Enter the header key.
                                            </FormDescription>
                                            <FormControl>
                                                <Input {...field} />
                                            </FormControl>
                                            <FormMessage />
                                        </FormItem>
                                    )}
                                />
                                <FormField
                                    control={form.control}
                                    name={`http.custom_headers.${index}.value`}
                                    render={({ field }) => (
                                        <FormItem className="flex-1">
                                            <FormLabel className={cn(index !== 0 && "sr-only")}>Value:</FormLabel>
                                            <FormDescription className={cn(index !== 0 && "sr-only")}>
                                                Enter the header value.
                                            </FormDescription>
                                            <FormControl>
                                                <Input {...field} />
                                            </FormControl>
                                            <FormMessage />
                                        </FormItem>
                                    )}
                                />
                            </div>
                            <Button
                                type="button"
                                variant="ghost"
                                size="icon"
                                onClick={() => remove(index)}
                                className={cn(
                                    "text-red-500 hover:text-red-700 sm:self-center",
                                    index === 0 && "sm:mt-14"
                                )}
                            >
                                <MinusCircle className="h-5 w-5" />
                            </Button>
                        </div>
                    ))}
                    <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        className="mt-2"
                        onClick={() => append({ key: "", value: "" })}
                    >
                        <Plus className="mr-2 h-4 w-4" /> Add HTTP Header
                    </Button>
                </div>
                <FormField
                    control={form.control}
                    name='use_proxy'
                    render={({ field }) => (
                        <FormItem>
                            <FormLabel className="flex items-center justify-between">Use Proxy(optional)</FormLabel>
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
                                Use a SOCKS5 proxy for webhook connections.
                            </FormDescription>
                            <FormMessage />
                        </FormItem>
                    )}
                />
                {/* Watched Events */}
                <FormField
                    control={form.control}
                    name="watched_events"
                    render={({ field }) => (
                        <FormItem className="space-y-3">
                            <FormLabel>Watched Events:</FormLabel>
                            <FormControl>
                                <div className="space-y-2">
                                    <div className="flex gap-2 pb-2">
                                        <Button
                                            type="button"
                                            variant="outline"
                                            size="sm"
                                            onClick={() => {
                                                field.onChange([...eventTypeSchema.options]);
                                            }}
                                        >
                                            Select All
                                        </Button>
                                        <Button
                                            type="button"
                                            variant="outline"
                                            size="sm"
                                            onClick={() => {
                                                field.onChange([]);
                                            }}
                                        >
                                            Deselect All
                                        </Button>
                                    </div>

                                    {eventTypeOptions.map((option) => (
                                        <div key={option.value} className="flex items-start gap-x-3">
                                            <Checkbox
                                                id={`event-${option.value}`}
                                                checked={field.value?.includes(option.value)}
                                                onCheckedChange={(checked) => {
                                                    const currentValues = field.value || [];
                                                    field.onChange(
                                                        checked
                                                            ? [...currentValues, option.value]
                                                            : currentValues.filter((v) => v !== option.value)
                                                    );
                                                }}
                                            />
                                            <div className="grid gap-1.5 leading-none">
                                                <label htmlFor={`event-${option.value}`} className="text-sm font-medium">
                                                    {option.label}
                                                </label>
                                                {option.description && (
                                                    <p className="text-sm text-muted-foreground">
                                                        {option.description}
                                                    </p>
                                                )}
                                            </div>
                                        </div>
                                    ))}
                                </div>
                            </FormControl>
                            <FormDescription>
                                Select the events that will trigger this event hook.
                            </FormDescription>
                            <FormMessage />
                        </FormItem>
                    )}
                />

                {/* VRL Script Section */}
                <Accordion type="single" collapsible>
                    <AccordionItem value="item-1">
                        <AccordionTrigger className="font-semibold">Use VRL scripts for event filtering and payload format modification</AccordionTrigger>
                        <AccordionContent>
                            <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
                                {/* Left: VRL Script Editor */}
                                <div className="space-y-2 h-full">
                                    <FormField
                                        control={form.control}
                                        name='vrl_script'
                                        render={({ field }) => (
                                            <FormItem className='flex flex-col gap-y-1 space-y-0'>
                                                <FormLabel>VRL Script:</FormLabel>
                                                <FormControl>
                                                    <CodeEditorWithDraft
                                                        value={field.value}
                                                        onChange={field.onChange}
                                                        localStorageKey="draft_vrl_script"
                                                        mode="python"
                                                        placeholder='Enter the VRL script here ...'
                                                        theme={'monokai'}
                                                        className="w-full h-[50rem]"
                                                    />
                                                </FormControl>
                                                <FormDescription>
                                                    Define the VRL (Vector Remap Language) script that specifies the structure and transformation of the payload.
                                                    see <a
                                                        href="https://vector.dev/docs/reference/vrl/"
                                                        target="_blank"
                                                        rel="noopener noreferrer"
                                                        className="text-blue-600 hover:underline"
                                                    >
                                                        VRL Documentation
                                                    </a> and
                                                    <a
                                                        href="https://playground.vrl.dev/"
                                                        target="_blank"
                                                        rel="noopener noreferrer"
                                                        className="text-blue-600 hover:underline"
                                                    >
                                                        VRL Playground
                                                    </a>
                                                </FormDescription>
                                                <FormMessage />
                                            </FormItem>
                                        )}
                                    />
                                </div>

                                {/* Right: Example and Result Editors (stacked) */}
                                <div className="space-y-4">
                                    {/* Input JSON Editor */}
                                    <div className="space-y-2">
                                        <h3 className="text-sm font-medium">Event Examples:</h3>
                                        <Select
                                            onValueChange={(eventType) => {
                                                const example = eventExamples?.[eventType as string];
                                                setInputJson(example ? JSON.stringify(example, null, 2) : "");
                                                setResolveResult(undefined);
                                            }}
                                        >
                                            <SelectTrigger className="w-[20rem]">
                                                <SelectValue placeholder="Select an event example" />
                                            </SelectTrigger>
                                            <SelectContent>
                                                {eventExamples && Object.entries(eventExamples).map(([type, _]) => (
                                                    <SelectItem key={type} value={type}>
                                                        {type}
                                                    </SelectItem>
                                                ))}
                                            </SelectContent>
                                        </Select>
                                        <ScrollArea className="border rounded-md">
                                            <AceEditor
                                                placeholder='event input.'
                                                value={inputJson}
                                                onChange={setInputJson}
                                                className="h-[20rem]"
                                                mode='json'
                                                theme={'monokai'}
                                            />
                                        </ScrollArea>
                                    </div>

                                    {/* Output JSON Editor */}
                                    <div className="space-y-2">
                                        <h3 className="text-sm font-medium">Output JSON:</h3>
                                        <div className="flex space-x-2">
                                            <Button variant="outline" type="button" onClick={(e) => {
                                                e.preventDefault();
                                                e.stopPropagation();
                                                runTest()
                                            }}>
                                                Run Test
                                            </Button>
                                        </div>
                                        <ScrollArea className="border rounded-md">
                                            <AceEditor
                                                placeholder='VRL script test result will be here'
                                                value={resolveResult}
                                                className="h-[20rem]"
                                                mode='json'
                                                theme={'monokai'}
                                            />
                                        </ScrollArea>
                                    </div>
                                </div>
                            </div>
                        </AccordionContent>
                    </AccordionItem>
                </Accordion>

                {/* Description */}
                <FormField
                    control={form.control}
                    name='description'
                    render={({ field }) => (
                        <FormItem className='flex flex-col gap-y-1 space-y-0'>
                            <FormLabel className='mb-1'>Description:</FormLabel>
                            <FormControl>
                                <Textarea
                                    placeholder='Enter a description for the event hook (optional)'
                                    {...field}
                                    className='max-h-[240px] min-h-[100px]'
                                />
                            </FormControl>
                            <FormDescription>
                                Add a description to explain the purpose or usage of this event hook.
                            </FormDescription>
                            <FormMessage />
                        </FormItem>
                    )}
                />
            </form>
        </Form>
    )
}