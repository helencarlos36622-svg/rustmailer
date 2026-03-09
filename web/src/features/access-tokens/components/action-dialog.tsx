/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { z } from 'zod'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { toast } from '@/hooks/use-toast'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {
  Form,
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/components/ui/form'
import { Input } from '@/components/ui/input'
import { ScrollArea } from '@/components/ui/scroll-area'
// import { MultiSelect } from '@/components/multi-select'
import { Textarea } from '@/components/ui/textarea'
import { AccessToken } from '../data/schema'
import useMinimalAccountList from '@/hooks/use-minimal-account-list'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { create_access_token, update_access_token } from '@/api/access-tokens/api'
import { ToastAction } from '@/components/ui/toast'
import { AxiosError } from 'axios'
import { VirtualizedSelect } from '@/components/virtualized-select'
import { MultiSelect } from '@/components/multi-select'
import { Loader2 } from 'lucide-react'

const isValidIP = (ip: string) => {
  const ipv4Regex = /^(?:(?:\d{1,3}\.){3}\d{1,3})$/;
  const ipv6Regex = /^([0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}$/;
  return ipv4Regex.test(ip) || ipv6Regex.test(ip);
};


const accessTokenScopeSchema = z.enum(['Api', 'Metrics']);

const rateLimitSchema = z.object({
  quota: z.optional(z.number().int().positive({ message: "Quota must be a positive integer." })),
  interval: z.optional(z.number().int().positive({ message: "Interval must be a positive integer." })),
});

const accessControlSchema = z.object({
  ip_whitelist: z.string().optional(),
  rate_limit: rateLimitSchema.optional(),
}).transform((data) => {
  if (data.ip_whitelist) {
    const ips = data.ip_whitelist
      .split('\n')
      .map((ip) => ip.trim())
      .filter((ip) => ip !== '');

    return {
      ...data,
      ip_whitelist: ips.join('\n'),
    };
  }
  return data;
}).refine(
  (data) => {
    if (data.ip_whitelist) {
      const ips = data.ip_whitelist.split('\n');
      const invalidIPs = ips.filter((ip) => !isValidIP(ip));
      return invalidIPs.length === 0;
    }
    return true;
  },
  {
    message: 'Invalid IP addresses found. Please enter valid IPv4 or IPv6 addresses.',
    path: ['ip_whitelist'],
  }
).transform((data) => {
  if (data.rate_limit && !data.rate_limit.interval && !data.rate_limit.quota) {
    return {
      ...data,
      rate_limit: undefined,
    };
  }
  return data;
})
  .transform((data) => {
    if (!data.ip_whitelist && !data.rate_limit) {
      return undefined;
    }
    return data;
  });

const accessTokenFormSchema = z.object({
  accounts: z
    .array(z.number())
    .min(1, { message: "At least one account is required." }),
  description: z
    .optional(z.string().max(255, { message: "Description must not exceed 255 characters." })),
  access_scopes: z
    .array(accessTokenScopeSchema)
    .min(1, { message: "At least one access scope is required." }),
  acl: z.optional(accessControlSchema),
});

export type AccessTokenForm = z.infer<typeof accessTokenFormSchema>;


const accessTokenScopeOptions = [
  { value: 'Api', label: 'Api' },
  { value: 'Metrics', label: 'Metrics' }
];

interface Props {
  currentRow?: AccessToken
  open: boolean
  onOpenChange: (open: boolean) => void
}

const defaultValues = {
  accounts: [],
  description: undefined,
  access_scopes: [],
  acl: undefined,
};


export function TokensActionDialog({ currentRow, open, onOpenChange }: Props) {
  const isEdit = !!currentRow
  const queryClient = useQueryClient();
  const form = useForm<AccessTokenForm>({
    resolver: zodResolver(accessTokenFormSchema),
    defaultValues: isEdit
      ? {
        accounts: currentRow.accounts.map(value => value.id),
        access_scopes: currentRow.access_scopes,
        description: currentRow.description,
        acl: currentRow.acl
          ? {
            ip_whitelist: currentRow.acl.ip_whitelist
              ? currentRow.acl.ip_whitelist.join('\n')
              : undefined,
            rate_limit: currentRow.acl.rate_limit ? currentRow.acl.rate_limit : undefined
          }
          : undefined,
      }
      : defaultValues,
  });

  const createMutation = useMutation({
    mutationFn: create_access_token,
    onSuccess: handleSuccess,
    onError: handleError
  });

  const updateMutation = useMutation({
    mutationFn: (data: Record<string, any>) => update_access_token(currentRow?.token ?? '', data),
    onSuccess: handleSuccess,
    onError: handleError
  })

  function handleSuccess() {
    toast({
      title: `Access token ${isEdit ? 'Updated' : 'Created'}`,
      description: `Your access token has been successfully ${isEdit ? 'updated' : 'created'}.`,
      action: <ToastAction altText="Close">Close</ToastAction>,
    });

    queryClient.invalidateQueries({ queryKey: ['access-tokens'] });
    form.reset();
    onOpenChange(false);
  }

  function handleError(error: AxiosError) {
    const errorMessage = (error.response?.data as { message?: string })?.message ||
      error.message ||
      `${isEdit ? 'Update' : 'Creation'} failed, please try again later`;

    toast({
      variant: "destructive",
      title: `Access token ${isEdit ? 'Update' : 'Creation'} Failed`,
      description: errorMessage as string,
      action: <ToastAction altText="Try again">Try again</ToastAction>,
    });
    console.error(error);
  }

  const { accountsOptions, isLoading } = useMinimalAccountList();

  const onSubmit = (values: AccessTokenForm) => {
    const payload = {
      accounts: values.accounts,
      description: values.description,
      access_scopes: values.access_scopes,
      acl: values.acl
        ? {
          ...values.acl,
          ip_whitelist: values.acl.ip_whitelist
            ? (() => {
              const ipSet = new Set(
                values.acl.ip_whitelist
                  .split('\n')
                  .map(ip => ip.trim())
                  .filter(ip => ip !== ''),
              );
              return ipSet.size > 0 ? Array.from(ipSet) : undefined;
            })()
            : undefined,
        }
        : undefined,
    };

    if (isEdit) {
      updateMutation.mutate(payload);
    } else {
      createMutation.mutate(payload);
    }
  }

  return (
    <Dialog
      open={open}
      onOpenChange={(state) => {
        form.reset()
        onOpenChange(state)
      }}
    >
      <DialogContent className='max-w-4xl'>
        <DialogHeader className='text-left mb-4'>
          <DialogTitle>{isEdit ? 'Edit Token' : 'Add New Token'}</DialogTitle>
          <DialogDescription>
            {isEdit ? 'Update the access token here. ' : 'Create new access token here. '}
            Click save when you&apos;re done.
          </DialogDescription>
        </DialogHeader>
        <ScrollArea className='h-[45rem] w-full pr-4 -mr-4 py-1'>
          <Form {...form}>
            <form
              id='token-form'
              onSubmit={form.handleSubmit(onSubmit)}
              className='space-y-4 p-0.5'
            >
              <FormField
                control={form.control}
                name='accounts'
                render={({ field }) => (
                  <FormItem className='flex flex-col gap-y-1 space-y-0'>
                    <FormLabel className='mb-1'>Accounts:</FormLabel>
                    <FormControl>
                      <VirtualizedSelect
                        multiple
                        options={accountsOptions}
                        className='w-full'
                        isLoading={isLoading}
                        onSelectOption={(options) => {
                          const numberArray = options.map((v) => parseInt(v, 10));
                          return field.onChange(numberArray);
                        }}
                        defaultValue={`${field.value}`}
                        placeholder="Select accounts"
                      />
                    </FormControl>
                    <FormMessage />
                    <FormDescription>
                      Select multiple accounts for the access token's authorization scope.
                    </FormDescription>
                  </FormItem>
                )}
              />
              <FormField
                control={form.control}
                name='access_scopes'
                render={({ field }) => (
                  <FormItem className='flex flex-col gap-y-1 space-y-0'>
                    <FormLabel className='mb-1'>Scopes:</FormLabel>
                    <FormControl>
                      <MultiSelect
                        options={accessTokenScopeOptions}
                        onValueChange={field.onChange}
                        defaultValue={field.value}
                        placeholder="Select scopes"
                        variant="default"
                        animation={0}
                        maxCount={3}
                      />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <FormField
                control={form.control}
                name="acl.ip_whitelist"
                render={({ field }) => (
                  <FormItem className="flex flex-col gap-y-1 space-y-0">
                    <FormLabel className='mb-1'>IP Whitelist:</FormLabel>
                    <FormControl>
                      <Textarea
                        placeholder="Enter one IP address per line, e.g.:\n192.168.1.1\n192.168.1.2"
                        {...field}
                        className="max-h-[500px] min-h-[180px]"
                      />
                    </FormControl>
                    <FormDescription>
                      A list of IP addresses allowed to access the resource. Enter one IP address per line. (Optional)
                    </FormDescription>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <div className="flex gap-4">
                <FormField
                  control={form.control}
                  name="acl.rate_limit.quota"
                  render={({ field }) => (
                    <FormItem className="flex flex-col gap-y-1 space-y-0 w-1/2">
                      <FormLabel className='mb-1'>Quota:</FormLabel>
                      <FormControl>
                        <Input
                          type="number"
                          placeholder="Enter quota, e.g., 100"
                          {...field}
                          onChange={(e) => field.onChange(parseInt(e.target.value, 10))}
                        />
                      </FormControl>
                      <FormDescription>
                        The maximum number of requests allowed within the interval. (Optional)
                      </FormDescription>
                      <FormMessage />
                    </FormItem>
                  )}
                />
                <FormField
                  control={form.control}
                  name="acl.rate_limit.interval"
                  render={({ field }) => (
                    <FormItem className="flex flex-col gap-y-1 space-y-0 w-1/2">
                      <FormLabel className='mb-1'>Interval (seconds):</FormLabel>
                      <FormControl>
                        <Input
                          type="number"
                          placeholder="Enter interval in seconds, e.g., 60"
                          {...field}
                          onChange={(e) => field.onChange(parseInt(e.target.value, 10))}
                        />
                      </FormControl>
                      <FormDescription>
                        The time window (in seconds) for the rate limit. (Optional)
                      </FormDescription>
                      <FormMessage />
                    </FormItem>
                  )}
                />
              </div>
              <FormField
                control={form.control}
                name='description'
                render={({ field }) => (
                  <FormItem className='flex flex-col gap-y-1 space-y-0'>
                    <FormLabel className='mb-1'>Description:</FormLabel>
                    <FormControl>
                      <Textarea
                        placeholder='Describe the purpose of the access token'
                        {...field}
                        className="max-h-[240px] min-h-[80px]"
                      />
                    </FormControl>
                    <FormDescription>(Optional)</FormDescription>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </form>
          </Form>
        </ScrollArea>
        <DialogFooter>
          <Button
            type="submit"
            form="token-form"
            disabled={isEdit ? updateMutation.isPending : createMutation.isPending}
            className="min-w-[100px] relative transition-all"
          >
            <span className="inline-flex items-center justify-center gap-2">
              {(isEdit ? updateMutation.isPending : createMutation.isPending) && (
                <Loader2 className="h-4 w-4 animate-spin" />
              )}
              <span>
                {isEdit
                  ? updateMutation.isPending
                    ? "Updating..."
                    : "Save changes"
                  : createMutation.isPending
                    ? "Creating..."
                    : "Save"}
              </span>
            </span>
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
