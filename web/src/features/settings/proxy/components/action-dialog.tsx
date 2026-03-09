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
import { Proxy } from '../data/schema'
import { AxiosError } from 'axios'
import { ToastAction } from '@/components/ui/toast'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { Loader2 } from 'lucide-react'
import { add_proxy, update_proxy } from '@/api/system/api'


const proxyFormSchema = z.object({
  url: z.string()
    .min(1, "Proxy address cannot be empty")
    .refine(
      (value) => {
        try {
          const url = new URL(value);
          return url.protocol === 'socks5:' || url.protocol === 'http:';
        } catch {
          return false;
        }
      },
      {
        message: "URL must start with http:// or socks5://",
      }
    )
    .refine(
      (value) => {
        const url = new URL(value);
        return /^[a-zA-Z0-9\-\.]+$/.test(url.hostname);
      },
      {
        message: "Hostname contains invalid characters",
      }
    )
    .refine(
      (value) => {
        const url = new URL(value);
        const port = parseInt(url.port || '1080');
        return port > 0 && port <= 65535;
      },
      {
        message: "Port must be between 1-65535",
      }
    )
    .refine(
      (value) => {
        const url = new URL(value);
        if (url.username && !url.password) return false;
        return true;
      },
      {
        message: "Password cannot be empty when username is provided",
      }
    )
    .refine(
      (value) => {
        const url = new URL(value);
        if (url.password) return url.password.length >= 8;
        return true;
      },
      {
        message: "Password must be at least 8 characters",
      }
    )
});

export type ProxyForm = z.infer<typeof proxyFormSchema>;


interface Props {
  currentRow?: Proxy
  open: boolean
  onOpenChange: (open: boolean) => void
}

const defaultValues = {
  url: ""
};


const mapCurrentRowToFormValues = (currentRow: Proxy) => {
  let data = {
    url: currentRow.url
  };
  return data;
};

export function ProxyActionDialog({ currentRow, open, onOpenChange }: Props) {
  const isEdit = !!currentRow
  const queryClient = useQueryClient();
  const form = useForm<ProxyForm>({
    resolver: zodResolver(proxyFormSchema),
    defaultValues: isEdit
      ? mapCurrentRowToFormValues(currentRow)
      : defaultValues,
  });


  const createMutation = useMutation({
    mutationFn: add_proxy,
    onSuccess: handleSuccess,
    onError: handleError
  });

  const updateMutation = useMutation({
    mutationFn: (url: string) => update_proxy(currentRow?.id!, url),
    onSuccess: handleSuccess,
    onError: handleError
  })

  function handleSuccess() {
    toast({
      title: `Proxy ${isEdit ? 'Updated' : 'Added'}`,
      description: `Your Proxy has been successfully ${isEdit ? 'updated' : 'added'}.`,
      action: <ToastAction altText="Close">Close</ToastAction>,
    });

    queryClient.invalidateQueries({ queryKey: ['proxy-list'] });
    form.reset();
    onOpenChange(false);
  }

  function handleError(error: AxiosError) {
    const errorMessage = (error.response?.data as { message?: string })?.message ||
      error.message ||
      `${isEdit ? 'Update' : 'Add'} failed, please try again later`;

    toast({
      variant: "destructive",
      title: `Proxy ${isEdit ? 'Update' : 'Add'} Failed`,
      description: errorMessage as string,
      action: <ToastAction altText="Try again">Try again</ToastAction>,
    });
    console.error(error);
  }


  const onSubmit = (values: ProxyForm) => {
    const url = values.url;
    if (isEdit) {
      updateMutation.mutate(url);
    } else {
      createMutation.mutate(url);
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
      <DialogContent className='max-w-xl'>
        <DialogHeader className='text-left mb-4'>
          <DialogTitle>{isEdit ? 'Edit Proxy' : 'Add New Proxy'}</DialogTitle>
          <DialogDescription>
            {isEdit ? 'Update the Proxy here. ' : 'Add new Proxy here. '}
            Click save when you&apos;re done.
          </DialogDescription>
        </DialogHeader>
        <Form {...form}>
          <form
            id='proxy-form'
            onSubmit={form.handleSubmit(onSubmit)}
            className='space-y-4 p-0.5'
          >
            <FormField
              control={form.control}
              name="url"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Proxy URL</FormLabel>
                  <FormControl>
                    <Input
                      placeholder="socks5://127.0.0.1:22308"
                      {...field}
                    />
                  </FormControl>
                  <FormMessage />
                  <FormDescription>
                    Please use an IP address (e.g., 127.0.0.1) rather than a hostname or domain for better reliability.
                  </FormDescription>
                </FormItem>
              )}
            />
          </form>
        </Form>
        <DialogFooter>
          <Button
            type="submit"
            form="proxy-form"
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
                    ? "Adding..."
                    : "Add"}
              </span>
            </span>
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
