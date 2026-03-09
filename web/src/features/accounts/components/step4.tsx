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
import { useFormContext } from "react-hook-form";
import { Account } from "./action-dialog";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { Button } from "@/components/ui/button";
import { format } from "date-fns";
import { CalendarIcon } from "lucide-react";
import { Calendar } from "@/components/ui/calendar";
import { cn } from "@/lib/utils";
import { RadioGroup, RadioGroupItem } from "@/components/ui/radio-group";
import { useState } from "react";
import { Checkbox } from "@/components/ui/checkbox";

interface StepProps {
    isEdit: boolean;
}

export default function Step4({ isEdit }: StepProps) {
    const { control, getValues, setValue } = useFormContext<Account>();
    const current = getValues();
    const [rangeType, setRangeType] = useState<'none' | 'fixed' | 'relative'>(current.date_since ? (current.date_since.fixed ? 'fixed' : 'relative') : 'none')

    return (
        <>
            <div className="space-y-8">
                <FormField
                    control={control}
                    name="full_sync_interval_min"
                    render={({ field }) => (
                        <FormItem>
                            <FormLabel className="flex items-center justify-between">
                                Full Sync(minutes):
                            </FormLabel>
                            <FormControl>
                                <Input type="number" placeholder="e.g 60" {...field} onChange={(e) => field.onChange(parseInt(e.target.value, 10))} />
                            </FormControl>
                            <FormMessage />
                        </FormItem>
                    )}
                />
                <FormField
                    control={control}
                    name="incremental_sync_interval_sec"
                    render={({ field }) => (
                        <FormItem>
                            <FormLabel className="flex items-center justify-between">
                                Incremental Sync(seconds):
                            </FormLabel>
                            <FormControl>
                                <Input type="number" placeholder="e.g 300" {...field} onChange={(e) => field.onChange(parseInt(e.target.value, 10))} />
                            </FormControl>
                            <FormMessage />
                        </FormItem>
                    )}
                />
                <FormField
                    control={control}
                    name='enabled'
                    render={({ field }) => (
                        <FormItem className='flex flex-col items-start gap-y-1'>
                            <FormLabel>Enabled:</FormLabel>
                            <FormControl>
                                <Checkbox
                                    className='mt-2'
                                    checked={field.value}
                                    onCheckedChange={field.onChange}
                                />
                            </FormControl>
                            <FormDescription>
                                Determines whether this account is active. If disabled, related syncs and queries will not run.
                            </FormDescription>
                        </FormItem>
                    )}
                />
                <FormLabel className="flex items-center justify-between">
                    Date Since:
                </FormLabel>
                <RadioGroup
                    defaultValue={rangeType}
                    onValueChange={(value: 'fixed' | 'relative' | 'none') => {
                        setRangeType(value);
                        if (value === 'none') {
                            setValue("date_since", undefined, { shouldValidate: true });
                        }

                        if (value === 'fixed') {
                            setValue("date_since", { fixed: undefined }, { shouldValidate: true });
                        }

                        if (value === 'relative') {
                            setValue("date_since", { relative: { value: undefined, unit: undefined } }, { shouldValidate: true });
                        }
                    }}
                    className='flex flex-row space-x-4'
                >
                    <FormItem className='flex items-center space-x-3'>
                        <RadioGroupItem value='none' />
                        <FormLabel className='font-normal'>None</FormLabel>
                    </FormItem>
                    <FormItem className='flex items-center space-x-3'>
                        <RadioGroupItem value='fixed' />
                        <FormLabel className='font-normal'>Fixed</FormLabel>
                    </FormItem>
                    <FormItem className='flex items-center space-x-3'>
                        <RadioGroupItem value='relative' />
                        <FormLabel className='font-normal'>Relative</FormLabel>
                    </FormItem>
                </RadioGroup>
                <FormDescription>defines the sync start date—either specific or relative to now. Preceding emails are excluded,{rangeType === 'fixed' ? " syncs data after a set date" : " shifts the sync date over time, syncing only recent data."}</FormDescription>
                {rangeType === 'fixed' && <FormField
                    control={control}
                    name="date_since.fixed"
                    render={({ field }) => (
                        <FormItem className="flex flex-col">
                            <Popover>
                                <PopoverTrigger asChild>
                                    <FormControl>
                                        <Button
                                            variant={"outline"}
                                            className={cn(
                                                "w-[240px] pl-3 text-left font-normal text-sm text-brand-marine-blue",
                                                !field.value && "text-muted-foreground"
                                            )}
                                        >
                                            {field.value ? (
                                                format(field.value, "PPP")
                                            ) : (
                                                <span>Pick a date</span>
                                            )}
                                            <CalendarIcon className="ml-auto h-4 w-4 opacity-50" />
                                        </Button>
                                    </FormControl>
                                </PopoverTrigger>
                                <PopoverContent className="w-auto p-0" align="start">
                                    <Calendar
                                        mode="single"
                                        selected={field.value ? new Date(new Date(field.value).setHours(0, 0, 0, 0)) : undefined}
                                        onSelect={(value) => {
                                            if (value) {
                                                const formattedDate = value.toLocaleDateString('en-CA')
                                                field.onChange(formattedDate)
                                            } else {
                                                field.onChange(null)
                                            }
                                        }}
                                        disabled={(date) =>
                                            date > new Date() || date < new Date("1900-01-01")
                                        }
                                        initialFocus
                                    />
                                </PopoverContent>
                            </Popover>
                            <FormMessage />
                        </FormItem>
                    )}
                />}
                {rangeType === 'relative' && <div className="flex flex-row gap-4">
                    <div className="flex-1">
                        <FormField
                            control={control}
                            name="date_since.relative.value"
                            render={({ field }) => (
                                <FormItem>
                                    <FormControl>
                                        <Input type="number" placeholder="e.g 1" {...field} onChange={(e) => field.onChange(parseInt(e.target.value, 10))} />
                                    </FormControl>
                                    <FormMessage />
                                </FormItem>
                            )}
                        />
                    </div>
                    <div className="w-1/2">
                        <FormField
                            control={control}
                            name="date_since.relative.unit"
                            render={({ field }) => (
                                <FormItem>
                                    <Select onValueChange={field.onChange} defaultValue={field.value}>
                                        <FormControl>
                                            <SelectTrigger>
                                                <SelectValue placeholder="Select unit" />
                                            </SelectTrigger>
                                        </FormControl>
                                        <SelectContent>
                                            <SelectItem value="Days">Days</SelectItem>
                                            <SelectItem value="Months">Months</SelectItem>
                                            <SelectItem value="Years">Years</SelectItem>
                                        </SelectContent>
                                    </Select>
                                    <FormMessage />
                                </FormItem>
                            )}
                        />
                    </div>
                </div>}
                <FormField
                    control={control}
                    name='minimal_sync'
                    render={({ field }) => (
                        <FormItem className='flex flex-col items-start gap-y-1'>
                            <FormLabel>Minimal Sync:</FormLabel>
                            <FormControl>
                                <Checkbox
                                    className='mt-2'
                                    checked={field.value}
                                    onCheckedChange={isEdit ? undefined : field.onChange}
                                    disabled={isEdit}
                                />
                            </FormControl>
                            <FormDescription>
                                {isEdit ? (
                                    "This setting cannot be modified after account creation."
                                ) : (
                                    "Syncing only essential basic metadata fields without building a local cache of email metadata, offering higher sync efficiency."
                                )}
                            </FormDescription>
                        </FormItem>
                    )}
                />

                <FormField
                    control={control}
                    name="folder_limit"
                    render={({ field }) => (
                        <FormItem>
                            <FormLabel className="flex items-center justify-between">
                                Folder Sync Limit:
                            </FormLabel>
                            <FormDescription>
                                Limit the number of emails to sync per folder (minimum 100). Leave empty for no limit.
                            </FormDescription>
                            <FormControl>
                                <Input
                                    type="number"
                                    placeholder="e.g. 1000"
                                    {...field}
                                    onChange={(e) =>
                                        field.onChange(e.target.value ? parseInt(e.target.value, 10) : undefined)
                                    }
                                />
                            </FormControl>
                            <FormMessage />
                        </FormItem>
                    )}
                />
            </div>
        </>
    );
}