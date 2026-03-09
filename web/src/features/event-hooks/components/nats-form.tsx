/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { z } from 'zod'
import { useForm } from 'react-hook-form'
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
import { VirtualizedSelect } from '@/components/virtualized-select'
import { eventTypeSchema, eventTypeOptions } from './types'
import { Switch } from '@/components/ui/switch'
import { CodeEditorWithDraft } from '@/components/editor-with-draft'

export const natsAuthTypeSchema = z.enum(["None", "Token", "Password"]);

export const natsConfigSchema = z.object({
    host: z.string({
        required_error: 'Please provide a host.',
    }).min(1, { message: "Please provide a host" }),
    port: z.number({ invalid_type_error: 'Please provide a port.' }).int().min(0).max(65535, { message: 'port must be between 0 and 65535.' }),
    auth_type: natsAuthTypeSchema.default("None"),
    token: z.string({ required_error: "Please provide a token." }).optional(),
    username: z.string().optional(),
    password: z.string().optional(),
    stream_name: z.string({
        required_error: 'Please provide a stream name.',
    }).min(1, { message: 'Please provide a stream name.' }),
    namespace: z.string({
        required_error: 'Please provide a namespace.',
    }).min(1, { message: 'Please provide a namespace.' }).regex(/^[a-zA-Z][a-zA-Z0-9_]*$/, 'Invalid namespace format'),
});

export const natsFormSchema = z.object({
    account_id: z.number().optional(),
    description: z.optional(
        z.string().max(255, { message: "Description must not exceed 255 characters." })
    ),
    enabled: z.boolean(),
    global: z.boolean(),
    nats: natsConfigSchema,
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

export type NatsEventHookForm = z.infer<typeof natsFormSchema>

interface NatsFormProps {
    form: ReturnType<typeof useForm<NatsEventHookForm>>;
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

export function NatsForm({
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
}: NatsFormProps) {

    const isGlobal = form.watch('global');

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
                                    disabled={isUpdate}
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

                {/* NATS Configuration */}
                <FormField
                    control={form.control}
                    name='nats.host'
                    render={({ field }) => (
                        <FormItem className='flex flex-col gap-y-1 space-y-0'>
                            <FormLabel className='mb-1'>Host:</FormLabel>
                            <FormControl>
                                <Input
                                    placeholder='Enter the NATS server host (e.g., nats.example.com)'
                                    {...field}
                                />
                            </FormControl>
                            <FormDescription>
                                The hostname or IP address of the NATS server.
                            </FormDescription>
                            <FormMessage />
                        </FormItem>
                    )}
                />

                <FormField
                    control={form.control}
                    name='nats.port'
                    render={({ field }) => (
                        <FormItem className='flex flex-col gap-y-1 space-y-0'>
                            <FormLabel className='mb-1'>Port:</FormLabel>
                            <FormControl>
                                <Input
                                    type="number"
                                    placeholder="Enter the NATS server port (e.g., 4222)"
                                    {...field}
                                    onChange={(e) => field.onChange(parseInt(e.target.value, 10))}
                                />
                            </FormControl>
                            <FormDescription>
                                The port number of the NATS server (default is 4222).
                            </FormDescription>
                            <FormMessage />
                        </FormItem>
                    )}
                />

                <FormField
                    control={form.control}
                    name='nats.stream_name'
                    render={({ field }) => (
                        <FormItem className='flex flex-col gap-y-1 space-y-0'>
                            <FormLabel className='mb-1'>Stream Name:</FormLabel>
                            <FormControl>
                                <Input
                                    placeholder='Enter the NATS stream name'
                                    {...field}
                                />
                            </FormControl>
                            <FormDescription>
                                The name of the NATS stream where events will be published.
                            </FormDescription>
                            <FormMessage />
                        </FormItem>
                    )}
                />

                <FormField
                    control={form.control}
                    name='nats.namespace'
                    render={({ field }) => (
                        <FormItem className='flex flex-col gap-y-1 space-y-0'>
                            <FormLabel className='mb-1'>Namespace:</FormLabel>
                            <FormControl>
                                <Input
                                    placeholder='Enter the NATS namespace'
                                    {...field}
                                />
                            </FormControl>
                            <FormDescription>
                                The namespace is used as a prefix for NATS subjects to organize and isolate message streams.
                            </FormDescription>
                            <FormMessage />
                        </FormItem>
                    )}
                />

                {/* NATS Authentication */}
                <FormField
                    control={form.control}
                    name="nats.auth_type"
                    render={({ field }) => (
                        <FormItem className="space-y-1">
                            <FormLabel>Authentication Type:</FormLabel>
                            <Select onValueChange={field.onChange} defaultValue={field.value}>
                                <FormControl>
                                    <SelectTrigger>
                                        <SelectValue placeholder="Select authentication type" />
                                    </SelectTrigger>
                                </FormControl>
                                <SelectContent>
                                    <SelectItem value="None">None (No Authentication)</SelectItem>
                                    <SelectItem value="Token">Token Authentication</SelectItem>
                                    <SelectItem value="Password">Username/Password Authentication</SelectItem>
                                </SelectContent>
                            </Select>
                            <FormDescription>
                                Select the authentication method for NATS connection
                            </FormDescription>
                            <FormMessage />
                        </FormItem>
                    )}
                />

                {/* Token Authentication */}
                {form.watch('nats.auth_type') === 'Token' && (
                    <FormField
                        control={form.control}
                        name='nats.token'
                        render={({ field }) => (
                            <FormItem className='flex flex-col gap-y-1 space-y-0'>
                                <FormLabel className='mb-1'>Token:</FormLabel>
                                <FormControl>
                                    <Input
                                        placeholder='Enter the NATS authentication token'
                                        {...field}
                                    />
                                </FormControl>
                                <FormDescription>
                                    The authentication token for the NATS server
                                </FormDescription>
                                <FormMessage />
                            </FormItem>
                        )}
                    />
                )}

                {/* Password Authentication */}
                {form.watch('nats.auth_type') === 'Password' && (
                    <>
                        <FormField
                            control={form.control}
                            name='nats.username'
                            render={({ field }) => (
                                <FormItem className='flex flex-col gap-y-1 space-y-0'>
                                    <FormLabel className='mb-1'>Username:</FormLabel>
                                    <FormControl>
                                        <Input
                                            placeholder='Enter the NATS username'
                                            {...field}
                                        />
                                    </FormControl>
                                    <FormDescription>
                                        The username for authenticating with the NATS server
                                    </FormDescription>
                                    <FormMessage />
                                </FormItem>
                            )}
                        />
                        <FormField
                            control={form.control}
                            name='nats.password'
                            render={({ field }) => (
                                <FormItem className='flex flex-col gap-y-1 space-y-0'>
                                    <FormLabel className='mb-1'>Password:</FormLabel>
                                    <FormControl>
                                        <Input
                                            type="password"
                                            placeholder='Enter the NATS password'
                                            {...field}
                                        />
                                    </FormControl>
                                    <FormDescription>
                                        The password for authenticating with the NATS server
                                    </FormDescription>
                                    <FormMessage />
                                </FormItem>
                            )}
                        />
                    </>
                )}
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
                        <AccordionTrigger className="font-semibold">Use vrl scripts for event filtering and payload format modification</AccordionTrigger>
                        <AccordionContent>
                            <FormField
                                control={form.control}
                                name='vrl_script'
                                render={({ field }) => (
                                    <FormItem className='flex flex-col gap-y-1 space-y-0'>
                                        <div className="flex items-center justify-between mb-1">
                                            <FormLabel>VRL Script:</FormLabel>
                                        </div>
                                        <FormControl>
                                            <CodeEditorWithDraft
                                                value={field.value}
                                                onChange={field.onChange}
                                                localStorageKey="draft_vrl_script"
                                                mode="python"
                                                placeholder='Enter the VRL script here ...'
                                                theme={'monokai'}
                                                className="h-[40rem]"
                                            />
                                        </FormControl>
                                        <FormDescription>
                                            Define the VRL (Vector Remap Language) script that specifies the structure and transformation of the payload.
                                            <a
                                                href="https://vector.dev/docs/reference/vrl/"
                                                target="_blank"
                                                rel="noopener noreferrer"
                                                className="text-blue-600 hover:underline"
                                            >
                                                VRL Documentation
                                            </a>
                                        </FormDescription>
                                        <FormMessage />
                                    </FormItem>
                                )}
                            />

                            <div className="grid grid-cols-1 lg:grid-cols-2 gap-4 mt-6">
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
                                            placeholder='Enter the VRL script here'
                                            value={inputJson}
                                            onChange={setInputJson}
                                            className="h-[34rem]"
                                            mode='json'
                                            theme={'monokai'}
                                        />
                                    </ScrollArea>
                                </div>

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
                                            className="h-[34rem]"
                                            mode='json'
                                            theme={'monokai'}
                                        />
                                    </ScrollArea>
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