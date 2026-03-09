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
import { Textarea } from '@/components/ui/textarea'
import { MTARecord } from '../data/schema'
import { SelectDropdown } from '@/components/select-dropdown'
import { AxiosError } from 'axios'
import { ToastAction } from '@/components/ui/toast'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { create_mta, update_mta } from '@/api/mta/api'
import { Checkbox } from '@/components/ui/checkbox'
import { PasswordInput } from '@/components/password-input'
import { Loader2 } from 'lucide-react'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import useProxyList from '@/hooks/use-proxy'

const credentialsSchema = z.object({
  username: z
    .string({
      required_error: 'Please enter username.',
    })
    .min(1, { message: "MTA username is required." }),
  password: z
    .string().optional(),
})

const serverSchema = z.object({
  host: z
    .string({
      required_error: 'Please enter MTA host.',
    })
    .min(1, { message: "MTA host is required." }),

  port: z
    .number({
      required_error: "Please enter MTA port.",
    })
    .min(1, { message: "Port must be greater than 1." })
    .max(65535, { message: "Port cannot exceed 65535." })
    .refine(
      (port) => ![0, 80, 443].includes(port),
      {
        message: "Port 0, 80, and 443 are reserved and cannot be used.",
      }
    ),
  encryption: z
    .string({
      required_error: 'Please select MTA encryption type.',
    })
    .min(1, { message: "MTA encryption type is required." })
})

const mtaFormSchema = z.object({
  description: z.optional(
    z.string().max(255, { message: "Description must not exceed 255 characters." })
  ),
  credentials: credentialsSchema,
  server: serverSchema,
  dsn_capable: z.boolean(),
  use_proxy: z.number().optional()
});

export type MTAForm = z.infer<typeof mtaFormSchema>;

const encryptionOptions = [
  { value: 'StartTls', label: 'StartTLS' },
  { value: 'None', label: 'None' },
  { value: 'Ssl', label: 'Ssl' },
];

interface Props {
  currentRow?: MTARecord
  open: boolean
  onOpenChange: (open: boolean) => void
}

const defaultValues = {
  credentials: {
    username: "",
    password: ""
  },
  description: undefined,
  server: {
    host: '',
    port: 465,
    encryption: 'StartTls',
  },
  dsn_capable: false,
  use_proxy: undefined
};


const mapCurrentRowToFormValues = (currentRow: MTARecord) => {
  let data = {
    credentials: {
      username: currentRow.credentials.username,
      password: undefined,
    },
    server: currentRow.server,
    dsn_capable: currentRow.dsn_capable,
    description: currentRow.description ?? undefined,
    use_proxy: currentRow.use_proxy === null ? undefined : currentRow.use_proxy
  };
  return data;
};

export function MTAActionDialog({ currentRow, open, onOpenChange }: Props) {
  const isEdit = !!currentRow
  const queryClient = useQueryClient();
  const form = useForm<MTAForm>({
    resolver: zodResolver(mtaFormSchema),
    defaultValues: isEdit
      ? mapCurrentRowToFormValues(currentRow)
      : defaultValues,
  });
  const { proxyOptions } = useProxyList();

  const createMutation = useMutation({
    mutationFn: create_mta,
    onSuccess: handleSuccess,
    onError: handleError
  });

  const updateMutation = useMutation({
    mutationFn: (data: Record<string, any>) => update_mta(currentRow?.id!, data),
    onSuccess: handleSuccess,
    onError: handleError
  })

  function handleSuccess() {
    toast({
      title: `MTA ${isEdit ? 'Updated' : 'Created'}`,
      description: `Your MTA has been successfully ${isEdit ? 'updated' : 'created'}.`,
      action: <ToastAction altText="Close">Close</ToastAction>,
    });

    queryClient.invalidateQueries({ queryKey: ['MTA-list'] });
    form.reset();
    onOpenChange(false);
  }

  function handleError(error: AxiosError) {
    const errorMessage = (error.response?.data as { message?: string })?.message ||
      error.message ||
      `${isEdit ? 'Update' : 'Creation'} failed, please try again later`;

    toast({
      variant: "destructive",
      title: `MTA ${isEdit ? 'Update' : 'Creation'} Failed`,
      description: errorMessage as string,
      action: <ToastAction altText="Try again">Try again</ToastAction>,
    });
    console.error(error);
  }


  const onSubmit = (values: MTAForm) => {
    if (!isEdit) {
      if (!values.credentials.password) {
        form.setError('credentials.password', {
          type: 'manual',
          message: 'Credentials password is required'
        });
        return;
      }
      if (values.credentials.password.length < 1) {
        form.setError('credentials.password', {
          type: 'manual',
          message: 'Credentials password cannot be empty'
        });
        return;
      }
    }

    const prepareClientSecret = (secret: string | undefined) => {
      return secret && secret.trim() !== '' ? secret : undefined;
    };

    const payload = {
      description: values.description,
      credentials: {
        username: values.credentials.username,
        password: isEdit ? prepareClientSecret(values.credentials.password) : values.credentials.password!
      },
      server: values.server,
      dsn_capable: values.dsn_capable,
      use_proxy: values.use_proxy
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
      <DialogContent className='max-w-3xl'>
        <DialogHeader className='text-left mb-4'>
          <DialogTitle>{isEdit ? 'Edit MTA' : 'Add New MTA'}</DialogTitle>
          <DialogDescription>
            {isEdit ? 'Update the MTA here. ' : 'Create new MTA here. '}
            Click save when you&apos;re done.
          </DialogDescription>
        </DialogHeader>
        <ScrollArea className='h-[36rem] w-full pr-4 -mr-4 py-1'>
          <Form {...form}>
            <form
              id='mta-form'
              onSubmit={form.handleSubmit(onSubmit)}
              className='space-y-4 p-0.5'
            >
              <div className='space-y-6'>
                <div className="flex gap-4">
                  <FormField
                    control={form.control}
                    name='credentials.username'
                    render={({ field }) => (
                      <FormItem className='flex flex-col gap-y-1 space-y-0 w-1/2'>
                        <FormLabel className='mb-1'>Username:</FormLabel>
                        <FormControl>
                          <Input
                            placeholder='Enter the username for the MTA'
                            {...field}
                          />
                        </FormControl>
                        <FormDescription>
                          Used to authenticate with the MTA.
                        </FormDescription>
                        <FormMessage />
                      </FormItem>
                    )}
                  />
                  <FormField
                    control={form.control}
                    name='credentials.password'
                    render={({ field }) => (
                      <FormItem className='flex flex-col gap-y-1 space-y-0 w-1/2'>
                        <FormLabel className='mb-1'>Password:</FormLabel>
                        <FormControl>
                          <PasswordInput placeholder={isEdit ? "Leave empty to keep current password" : "Enter your password"} {...field} />
                        </FormControl>
                        <FormDescription>
                          {isEdit ? 'Leave empty to keep the existing secret. Only enter a new value if you want to change it.' : 'Used to authenticate with the MTA.'}
                        </FormDescription>
                        <FormMessage />
                      </FormItem>
                    )}
                  />
                </div>
                <div className="flex gap-4">
                  <FormField
                    control={form.control}
                    name='server.host'
                    render={({ field }) => (
                      <FormItem className='flex flex-col gap-y-1 space-y-0 w-1/2'>
                        <FormLabel className='mb-1'>Host:</FormLabel>
                        <FormControl>
                          <Input
                            placeholder='(e.g., mta.example.com)'
                            {...field}
                          />
                        </FormControl>
                        <FormDescription>
                          The hostname or IP address of the MTA.
                        </FormDescription>
                        <FormMessage />
                      </FormItem>
                    )}
                  />
                  <FormField
                    control={form.control}
                    name='server.port'
                    render={({ field }) => (
                      <FormItem className='flex flex-col gap-y-1 space-y-0 w-1/2'>
                        <FormLabel className='mb-1'>Port:</FormLabel>
                        <FormControl>
                          <Input
                            type="number"
                            placeholder="Enter the port number for the MTA (e.g., 587)"
                            {...field}
                            onChange={(e) => field.onChange(parseInt(e.target.value, 10))}
                          />
                        </FormControl>
                        <FormMessage />
                      </FormItem>
                    )}
                  />
                </div>
                <FormField
                  control={form.control}
                  name='server.encryption'
                  render={({ field }) => (
                    <FormItem className='flex flex-col gap-y-1 space-y-0'>
                      <FormLabel className='mb-1'>Encryption Type:</FormLabel>
                      <FormControl>
                        <SelectDropdown
                          defaultValue={field.value}
                          onValueChange={field.onChange}
                          placeholder='Select the encryption type for the MTA'
                          items={encryptionOptions}
                        />
                      </FormControl>
                      <FormMessage />
                    </FormItem>
                  )}
                />
                <FormField
                  control={form.control}
                  name='dsn_capable'
                  render={({ field }) => (
                    <FormItem className='flex flex-row items-center gap-x-2'>
                      <FormControl>
                        <Checkbox
                          className='mt-2'
                          checked={field.value}
                          onCheckedChange={field.onChange}
                        />
                      </FormControl>
                      <FormLabel>DSN</FormLabel>
                      <FormDescription>
                        Indicates whether supports Delivery Status Notifications (DSN)
                      </FormDescription>
                    </FormItem>
                  )}
                />
                <FormField
                  control={form.control}
                  name='use_proxy'
                  render={({ field }) => (
                    <FormItem>
                      <FormLabel>Use Proxy(optional):</FormLabel>
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
                        Use a SOCKS5 proxy for MTA connections.
                      </FormDescription>
                      <FormMessage />
                    </FormItem>
                  )}
                />
                <FormField
                  control={form.control}
                  name='description'
                  render={({ field }) => (
                    <FormItem className='flex flex-col gap-y-1 space-y-0'>
                      <FormLabel className='mb-1'>Description:</FormLabel>
                      <FormControl>
                        <Textarea
                          placeholder='Enter a description for the MTA (optional)'
                          {...field}
                          className='max-h-[240px] min-h-[140px]'
                        />
                      </FormControl>
                      <FormDescription>
                        Add a description to explain the purpose or usage of this MTA.
                      </FormDescription>
                      <FormMessage />
                    </FormItem>
                  )}
                />
              </div>
            </form>
          </Form>
        </ScrollArea>
        <DialogFooter>
          <Button
            type="submit"
            form="mta-form"
            disabled={isEdit ? updateMutation.isPending : createMutation.isPending}
            className="min-w-[120px] relative"
          >
            <span className="inline-flex items-center justify-center">
              {(isEdit ? updateMutation.isPending : createMutation.isPending) && (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              )}
              <span>
                {isEdit
                  ? updateMutation.isPending
                    ? "Saving..."
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
