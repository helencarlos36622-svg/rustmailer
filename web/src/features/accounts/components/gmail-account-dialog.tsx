/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { z } from 'zod';
import { Button } from '@/components/ui/button';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { ScrollArea } from '@/components/ui/scroll-area';
import { useToast } from '@/hooks/use-toast';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { ToastAction } from '@/components/ui/toast';
import { AxiosError } from 'axios';
import { AccountEntity, MailerType } from '../data/schema';
import React, { useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { create_account, update_account } from '@/api/account/api';
import { Form, FormControl, FormDescription, FormField, FormItem, FormLabel, FormMessage } from '@/components/ui/form';
import { Input } from '@/components/ui/input';
import { Checkbox } from '@/components/ui/checkbox';
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { Calendar } from '@/components/ui/calendar';
import { CalendarIcon, Loader2 } from 'lucide-react';
import { format } from 'date-fns';
import { cn } from '@/lib/utils';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import useProxyList from '@/hooks/use-proxy';


const relativeDateSchema = z.object({
  unit: z.enum(["Days", "Months", "Years"], { message: "Please select a unit" }),
  value: z.number({ message: 'Please enter a value' }).int().min(1, "Must be at least 1"),
});

const dateSelectionSchema = z.union([
  z.object({ fixed: z.string({ message: "Please select a date" }) },),
  z.object({ relative: relativeDateSchema }),
  z.undefined(),
]);


const accountSchema = () =>
  z.object({
    name: z.string().optional(),
    email: z.string({ required_error: 'Email is required' }).email({ message: 'Invalid email address' }),
    enabled: z.boolean(),
    minimal_sync: z.boolean(),
    use_proxy: z.number().optional(),
    date_since: dateSelectionSchema.optional(),
    folder_limit: z
      .number({ invalid_type_error: 'Folder limit must be a number' })
      .int()
      .min(100, { message: 'Folder limit must be at least 100' })
      .optional(),
    incremental_sync_interval_sec: z.number({ invalid_type_error: 'Incremental sync interval must be a number' }).int().min(1, { message: 'Incremental sync interval must be at least 1 second' }),
  });


export type GmailApiAccount = {
  name?: string;
  email: string;
  enabled: boolean;
  minimal_sync: boolean;
  date_since?: {
    fixed?: string;
    relative?: {
      unit?: 'Days' | 'Months' | 'Years';
      value?: number;
    };
  };
  folder_limit?: number,
  use_proxy?: number,
  incremental_sync_interval_sec: number;
};



interface Props {
  currentRow?: AccountEntity;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}


const defaultValues: GmailApiAccount = {
  name: '',
  email: '',
  enabled: true,
  date_since: undefined,
  folder_limit: undefined,
  incremental_sync_interval_sec: 30,
  minimal_sync: false,
  use_proxy: undefined
};


const mapCurrentRowToFormValues = (currentRow: AccountEntity): GmailApiAccount => {
  let account = {
    name: currentRow.name === null ? '' : currentRow.name,
    email: currentRow.email,
    enabled: currentRow.enabled,
    minimal_sync: currentRow.minimal_sync ?? false,
    date_since: currentRow.date_since ?? undefined,
    folder_limit: currentRow.folder_limit ?? undefined,
    incremental_sync_interval_sec: currentRow.incremental_sync_interval_sec,
    use_proxy: currentRow.use_proxy
  };
  return account;
};


export function GmailApiAccountDialog({ currentRow, open, onOpenChange }: Props) {
  const isEdit = !!currentRow;
  const { toast } = useToast();
  const [rangeType, setRangeType] = useState<'none' | 'fixed' | 'relative'>(currentRow?.date_since
    ? currentRow.date_since.fixed
      ? "fixed"
      : currentRow.date_since.relative
        ? "relative"
        : "none"
    : "none")

  const { proxyOptions } = useProxyList();

  const form = useForm<GmailApiAccount>({
    mode: "all",
    defaultValues: isEdit ? mapCurrentRowToFormValues(currentRow) : defaultValues,
    resolver: zodResolver(accountSchema()),
  });

  const queryClient = useQueryClient();

  const createMutation = useMutation({
    mutationFn: create_account,
    onSuccess: handleSuccess,
    onError: handleError,
  });

  const updateMutation = useMutation({
    mutationFn: (data: Record<string, any>) => update_account(currentRow?.id!, data),
    onSuccess: handleSuccess,
    onError: handleError,
  });

  function handleSuccess() {
    toast({
      title: `Account ${isEdit ? 'Updated' : 'Created'}`,
      description: `Your account has been successfully ${isEdit ? 'updated' : 'created'}.`,
      action: <ToastAction altText="Close">Close</ToastAction>,
    });

    queryClient.invalidateQueries({ queryKey: ['account-list'] });
    form.reset();
    onOpenChange(false);
  }

  function handleError(error: AxiosError) {
    const errorMessage =
      (error.response?.data as { message?: string })?.message ||
      error.message ||
      `${isEdit ? 'Update' : 'Creation'} failed, please try again later`;

    toast({
      variant: "destructive",
      title: `Account ${isEdit ? 'Update' : 'Creation'} Failed`,
      description: errorMessage as string,
      action: <ToastAction altText="Try again">Try again</ToastAction>,
    });
    console.error(error);
  }

  const onSubmit = React.useCallback(
    (data: GmailApiAccount) => {
      const commonData = {
        email: data.email,
        name: data.name,
        enabled: data.enabled,
        date_since: data.date_since,
        folder_limit: data.folder_limit,
        minimal_sync: data.minimal_sync,
        incremental_sync_interval_sec: data.incremental_sync_interval_sec,
        use_proxy: data.use_proxy
      };
      if (isEdit) {
        updateMutation.mutate(commonData);
      } else {
        const payload = {
          ...commonData,
          mailer_type: MailerType.GmailApi
        };
        createMutation.mutate(payload);
      }
    },
    [isEdit, updateMutation, createMutation]
  );
  return (
    <Dialog
      open={open}
      onOpenChange={(state) => {
        form.reset();
        onOpenChange(state);
      }}
    >
      <DialogContent className='max-w-4xl'>
        <DialogHeader className='text-left mb-4'>
          <DialogTitle>{isEdit ? "Update Account" : "Add Account"}</DialogTitle>
          <DialogDescription>
            {isEdit ? 'Update the email account here. ' : 'Add new email account here. '}
            Click save when you're done.
          </DialogDescription>
        </DialogHeader>
        <ScrollArea className='h-[40rem] w-full pr-4 -mr-4 py-1'>
          <Form {...form}>
            <form
              id='gmail-api-account-form'
              onSubmit={form.handleSubmit(onSubmit)}
              className='space-y-4 p-0.5'
            >
              <FormField
                control={form.control}
                name="email"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel className="flex items-center justify-between">
                      Email Address:
                    </FormLabel>
                    <FormControl>
                      <Input placeholder="e.g john.doe@gmail.com" readOnly={isEdit} {...field} />
                    </FormControl>
                    <FormMessage />
                    <FormDescription>{isEdit
                      ? "The email account address cannot be modified when editing."
                      : "Please enter a Google email accessible via Gmail API (e.g., @gmail.com or Google Workspace account)."}</FormDescription>
                  </FormItem>
                )}
              />
              <FormField
                control={form.control}
                name="name"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel className="flex items-center justify-between">
                      Name:
                    </FormLabel>
                    <FormControl>
                      <Input placeholder="e.g john.doe" {...field} />
                    </FormControl>
                    <FormDescription>Optional</FormDescription>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <FormField
                control={form.control}
                name="incremental_sync_interval_sec"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel className="flex items-center justify-between">
                      Incremental Sync(seconds):
                    </FormLabel>
                    <FormControl>
                      <Input type="number" placeholder="e.g 300" {...field} onChange={(e) => field.onChange(parseInt(e.target.value, 10))} />
                    </FormControl>
                    <FormDescription>
                      Set the interval (in seconds) for calling the Gmail History API for incremental sync. This determines how frequently updates are fetched for new or modified emails.
                    </FormDescription>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <FormField
                control={form.control}
                name='enabled'
                render={({ field }) => (
                  <FormItem className='flex flex-col items-start gap-y-1'>
                    <FormLabel>Enabled:</FormLabel>
                    <FormControl>
                      <Checkbox
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
              <FormField
                control={form.control}
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
                        "When enabled, Gmail metadata will not be cached locally, ensuring higher synchronization efficiency by syncing only essential basic metadata fields."
                      )}
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
                    form.setValue("date_since", undefined, { shouldValidate: true });
                  }

                  if (value === 'fixed') {
                    form.setValue("date_since", { fixed: undefined }, { shouldValidate: true });
                  }

                  if (value === 'relative') {
                    form.setValue("date_since", { relative: { value: undefined, unit: undefined } }, { shouldValidate: true });
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
                control={form.control}
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
                    control={form.control}
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
                    control={form.control}
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
                control={form.control}
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
              <FormField
                control={form.control}
                name='use_proxy'
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
                      Use a SOCKS5 proxy for Gmail API connections.
                    </FormDescription>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </form>
          </Form>
        </ScrollArea>
        <DialogFooter>
          <Button
            type='submit'
            form='gmail-api-account-form'
            disabled={isEdit ? updateMutation.isPending : createMutation.isPending}
          >
            {isEdit ? (
              updateMutation.isPending ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Saving...
                </>
              ) : (
                "Save changes"
              )
            ) : (
              createMutation.isPending ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Creating...
                </>
              ) : (
                "Create"
              )
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}