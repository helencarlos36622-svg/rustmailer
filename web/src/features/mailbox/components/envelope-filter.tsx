/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { Button } from '@/components/ui/button';
import { Calendar } from '@/components/ui/calendar';
import { Form, FormControl, FormField, FormItem, FormMessage } from '@/components/ui/form';
import { Input } from '@/components/ui/input';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { Select, SelectContent, SelectGroup, SelectItem, SelectLabel, SelectTrigger, SelectValue } from '@/components/ui/select';
import { cn } from '@/lib/utils';
import { zodResolver } from '@hookform/resolvers/zod'
import { CalendarIcon, Loader2, Plus, Trash2, X } from 'lucide-react';
import { useFieldArray, useForm } from 'react-hook-form';
import { z } from "zod";
import { format, parse } from 'date-fns';

const singleFields = ["Seen", "Unseen"] as const;

// Define the schema for a single condition
const conditionSchema = z.object({
    field: z.string().min(1, "Field cannot be empty"),
    operator: z.string().min(1, "Operator cannot be empty").optional(),
    value: z.string().min(1, "Value cannot be empty").optional()
}).superRefine((data, ctx) => {
    if (data.field === "To" || data.field === "Bcc" || data.field === "Cc" || data.field === "From") {
        if (!data.value) {
            ctx.addIssue({
                code: z.ZodIssueCode.custom,
                message: "Value cannot be empty",
                path: ["value"],
            });
        }
    } else if (data.field === "Subject" || data.field === "Text" || data.field === "Body") {
        if (!data.value) {
            ctx.addIssue({
                code: z.ZodIssueCode.custom,
                message: "Value cannot be empty",
                path: ["value"],
            });
        }
    } else if (data.field === "Before" || data.field === "Since" || data.field === "On" || data.field === "SentBefore" || data.field === "SentSince" || data.field === "SentOn") {
        if (!data.value) {
            ctx.addIssue({
                code: z.ZodIssueCode.custom,
                message: "Date cannot be empty",
                path: ["value"],
            });
        } else if (!z.string().regex(/^\d{4}-\d{2}-\d{2}$/).safeParse(data.value).success) {
            ctx.addIssue({
                code: z.ZodIssueCode.custom,
                message: "Date must be in yyyy-MM-dd format",
                path: ["value"],
            });
        }
    } else if (data.field === "Uid" || data.field === "Larger" || data.field === "Smaller") {
        if (!data.value) {
            ctx.addIssue({
                code: z.ZodIssueCode.custom,
                message: "Value cannot be empty",
                path: ["value"],
            });
        } else if (isNaN(Number(data.value))) {
            ctx.addIssue({
                code: z.ZodIssueCode.custom,
                message: "Value must be a number",
                path: ["value"],
            });
        } else if (Number(data.value) <= 0) {
            ctx.addIssue({
                code: z.ZodIssueCode.custom,
                message: "Value must be greater than 0",
                path: ["value"],
            });
        }
    } else if (!singleFields.includes(data.field as any)) {
        if (!data.operator) {
            ctx.addIssue({
                code: z.ZodIssueCode.custom,
                message: "Operator cannot be empty",
                path: ["operator"],
            });
        }
    }
});

// Define the schema for the logical combination (AND/OR) with only one level of conditions
const logicalConditionSchema = z.object({
    operator: z.enum(["and", "or"]).optional(),  // Logical operator to combine conditions (only one level of conditions)
    conditions: z.array(conditionSchema).min(1, "At least one condition is required").optional()  // Only one level of conditions
}).refine(
    (data) => {
        if (!data.conditions || data.conditions.length <= 1) {
            return true;
        }
        return data.operator !== undefined;
    },
    {
        message: "Operator is required when there are multiple conditions",
        path: ["operator"]
    }
);

export type FilterForm = z.infer<typeof logicalConditionSchema>

// const defaultValues: FilterForm = {
//     operator: undefined,
//     conditions: undefined,
// };
type Props = {
    handleSearch: (data: FilterForm) => Promise<void>;
    remote: boolean;
    isSearching: boolean;
    currentFilter?: FilterForm;
};

export function EnvelopeFilter({ handleSearch, remote, isSearching, currentFilter }: Props) {

    const filterForm = useForm<FilterForm>({
        resolver: zodResolver(logicalConditionSchema),
        defaultValues: currentFilter,
    })

    const { fields, append, remove } = useFieldArray({
        name: 'conditions',
        control: filterForm.control,
    })

    const onSubmit = async (data: FilterForm) => {
        await handleSearch(data)
    }

    const getValueInput = (index: number, fieldType: string) => {

        switch (fieldType) {
            case 'To':
            case 'Bcc':
            case 'Cc':
            case 'From':
                return (
                    <FormField
                        control={filterForm.control}
                        name={`conditions.${index}.value`}
                        render={({ field }) => (
                            <FormItem>
                                <FormControl>
                                    <Input {...field} value={field.value || ""} placeholder="e.g. example@example.com or example.com" />
                                </FormControl>
                                <FormMessage />
                            </FormItem>
                        )}
                    />
                );
            case 'Subject':
            case 'Text':
            case 'Body':
                return (
                    <FormField
                        control={filterForm.control}
                        name={`conditions.${index}.value`}
                        render={({ field }) => (
                            <FormItem>
                                <FormControl>
                                    <Input {...field} value={field.value || ""} placeholder="e.g. Hello World" />
                                </FormControl>
                                <FormMessage />
                            </FormItem>
                        )}
                    />
                );
            case 'Before':
            case 'Since':
            case 'On':
            case 'SentBefore':
            case 'SentSince':
            case 'SentOn':
                return (
                    <FormField
                        control={filterForm.control}
                        name={`conditions.${index}.value`}
                        render={({ field }) => {
                            const selectedDate = field.value && typeof field.value === 'string' ? parse(field.value, 'yyyy-MM-dd', new Date()) : undefined;
                            return (
                                <FormItem className='flex flex-col'>
                                    <Popover>
                                        <PopoverTrigger asChild>
                                            <FormControl>
                                                <Button
                                                    variant={'outline'}
                                                    className={cn(
                                                        'w-[240px] pl-3 text-left font-normal',
                                                        !field.value && 'text-muted-foreground'
                                                    )}
                                                >
                                                    {field.value ? (
                                                        format(field.value, 'yyyy-MM-dd')
                                                    ) : (
                                                        <span>Pick a date</span>
                                                    )}
                                                    <CalendarIcon className='ml-auto h-4 w-4 opacity-50' />
                                                </Button>
                                            </FormControl>
                                        </PopoverTrigger>
                                        <PopoverContent className='w-auto p-0' align='start'>
                                            <Calendar
                                                mode='single'
                                                selected={selectedDate}
                                                onSelect={(date) => {
                                                    if (date) {
                                                        const formattedDate = format(date, 'yyyy-MM-dd');
                                                        field.onChange(formattedDate);
                                                    } else {
                                                        field.onChange("");
                                                    }
                                                }}
                                                disabled={(date: Date) =>
                                                    date > new Date() || date < new Date('1900-01-01')
                                                }
                                            />
                                        </PopoverContent>
                                    </Popover>
                                    <FormMessage />
                                </FormItem>
                            )
                        }}
                    />
                );
            case 'Uid':
            case 'Larger':
            case 'Smaller':
                return (
                    <FormField
                        control={filterForm.control}
                        name={`conditions.${index}.value`}
                        render={({ field }) => (
                            <FormItem>
                                <FormControl>
                                    <Input {...field} type="number" placeholder="e.g. 60" value={field.value || ""} onChange={(e) => field.onChange(parseInt(e.target.value, 10))} />
                                </FormControl>
                                <FormMessage />
                            </FormItem>
                        )}
                    />
                );
            case 'Unseen':
            case 'Seen':
                return null; // No value input needed for these fields
            default:
                return null;
        }
    }

    return (
        <Form {...filterForm}>
            <form
                id='filter-form'
                onSubmit={filterForm.handleSubmit(onSubmit)}
                className='space-y-5 flex-1 mt-4'
            >
                <div className="space-y-2">
                    {fields.map((_, index) => (
                        <div className="grid grid-cols-1 sm:grid-cols-[auto_130px_80px_1fr_auto] gap-4 items-center" key={index}>
                            {index === 0 && <p className='text-gray-500 w-20 text-sm text-center'>Where</p>}
                            {index === 1 && (
                                <div className='text-center text-gray-500 w-20 text-sm'>
                                    <FormField
                                        control={filterForm.control}
                                        name="operator"
                                        render={({ field }) => (
                                            <FormItem>
                                                <Select onValueChange={field.onChange} defaultValue={field.value}>
                                                    <FormControl>
                                                        <SelectTrigger>
                                                            <SelectValue />
                                                        </SelectTrigger>
                                                    </FormControl>
                                                    <SelectContent>
                                                        <SelectItem value="and">and</SelectItem>
                                                        <SelectItem value="or">or</SelectItem>
                                                    </SelectContent>
                                                </Select>
                                                <FormMessage />
                                            </FormItem>
                                        )}
                                    />
                                </div>
                            )}
                            {index > 1 && <div className='text-center w-20 text-gray-500 text-sm'>{filterForm.getValues('operator')}</div>}
                            <FormField
                                control={filterForm.control}
                                name={`conditions.${index}.field`}
                                rules={{ required: "Select a field" }}
                                render={({ field }) => (
                                    <FormItem>
                                        <Select onValueChange={field.onChange} defaultValue={field.value}>
                                            <FormControl>
                                                <SelectTrigger>
                                                    <SelectValue placeholder="Select a field" />
                                                </SelectTrigger>
                                            </FormControl>
                                            <SelectContent>
                                                <SelectGroup>
                                                    <SelectLabel>Address</SelectLabel>
                                                    <SelectItem value="Bcc" title="Blind Carbon Copy address">bcc</SelectItem>
                                                    <SelectItem value="Cc" title="Carbon Copy address">cc</SelectItem>
                                                    <SelectItem value="From" title="Sender address">from</SelectItem>
                                                    <SelectItem value="To" title="Recipient address">to</SelectItem>
                                                </SelectGroup>

                                                {/* Received Date Group */}
                                                <SelectGroup>
                                                    <SelectLabel>Received Date</SelectLabel>
                                                    <SelectItem value="Before" title="Emails received before a specific date">before</SelectItem>
                                                    <SelectItem value="On" title="Emails received on a specific date">on</SelectItem>
                                                    <SelectItem value="Since" title="Emails received since a specific date">since</SelectItem>
                                                </SelectGroup>

                                                {/* Sent Date Group */}
                                                <SelectGroup>
                                                    <SelectLabel>Sent Date</SelectLabel>
                                                    <SelectItem value="SentBefore" title="Emails sent before a specific sent date">sent before</SelectItem>
                                                    <SelectItem value="SentOn" title="Emails sent on a specific sent date">set on</SelectItem>
                                                    <SelectItem value="SentSince" title="Emails sent since a specific sent date">sent since</SelectItem>
                                                </SelectGroup>

                                                {/* Read Group */}
                                                <SelectGroup>
                                                    <SelectLabel>Read</SelectLabel>
                                                    <SelectItem value="Seen" title="Emails that are read">seen</SelectItem>
                                                    <SelectItem value="Unseen" title="Emails that are unread">unseen</SelectItem>
                                                </SelectGroup>

                                                {/* Size Group */}
                                                <SelectGroup>
                                                    <SelectLabel>Size</SelectLabel>
                                                    <SelectItem value="Larger" title="Emails larger than a specified size">larger</SelectItem>
                                                    <SelectItem value="Smaller" title="Emails smaller than a specified size">smaller</SelectItem>
                                                </SelectGroup>

                                                {/* Subject or Body Group */}
                                                <SelectGroup>
                                                    <SelectLabel>Subject or Body</SelectLabel>
                                                    <SelectItem disabled={!remote} value="Body" title="Body content of the email">body</SelectItem>
                                                    <SelectItem value="Subject" title="Subject of the email">subject</SelectItem>
                                                    <SelectItem disabled={!remote} value="Text" title="Plain text content of the email">text</SelectItem>
                                                </SelectGroup>

                                                {/* UID Group */}
                                                <SelectGroup>
                                                    <SelectLabel>UID</SelectLabel>
                                                    <SelectItem value="Uid" title="Unique Identifier for the email">uid</SelectItem>
                                                </SelectGroup>
                                            </SelectContent>
                                        </Select>
                                        <FormMessage />
                                    </FormItem>
                                )}
                            />
                            {!singleFields.includes(filterForm.watch(`conditions.${index}.field`) as any) && <FormField
                                control={filterForm.control}
                                name={`conditions.${index}.operator`}
                                render={({ field }) => (
                                    <FormItem>
                                        <Select onValueChange={field.onChange} defaultValue={field.value}>
                                            <FormControl>
                                                <SelectTrigger>
                                                    <SelectValue />
                                                </SelectTrigger>
                                            </FormControl>
                                            <SelectContent className='text-center'>
                                                <SelectItem value="is">Is</SelectItem>
                                                <SelectItem value="is not">Is not</SelectItem>
                                            </SelectContent>
                                        </Select>
                                        <FormMessage />
                                    </FormItem>
                                )}
                            />}
                            {getValueInput(index, filterForm.watch(`conditions.${index}.field`))}
                            <Button
                                type="button"
                                variant="outline"
                                size="icon"
                                onClick={() => remove(index)}
                            >
                                <Trash2 className="h-5 w-5" />
                            </Button>
                        </div>
                    ))}
                    <div className='flex items-center space-x-4'>
                        <Button
                            type="button"
                            size="sm"
                            variant="outline"
                            className="mt-4"
                            onClick={() => append({ field: "", operator: "is", value: undefined })}
                        >
                            <Plus className="h-4 w-4" /> Add filter
                        </Button>
                        {fields.length > 0 && (
                            <Button
                                type="button"
                                variant="outline"
                                size="sm"
                                className="mt-4"
                                onClick={() => {
                                    filterForm.reset()
                                    filterForm.clearErrors()
                                    for (let i = 0; i < fields.length; i++) {
                                        remove(i)
                                    }
                                }}
                            >
                                <X className="h-4 w-4" /> Reset filters
                            </Button>
                        )}
                        <div className="flex flex-1 justify-end">
                            {fields.length > 0 && (
                                <Button
                                    form='filter-form'
                                    type='submit'
                                    size="sm"
                                    className="mt-4"
                                    disabled={isSearching}
                                >
                                    {isSearching ? (
                                        <div className="flex items-center gap-2">
                                            <Loader2 className="h-4 w-4 animate-spin" />
                                            Applying...
                                        </div>
                                    ) : "Apply"}
                                </Button>
                            )}
                        </div>
                    </div>
                </div>
            </form>
        </Form>
    )
}