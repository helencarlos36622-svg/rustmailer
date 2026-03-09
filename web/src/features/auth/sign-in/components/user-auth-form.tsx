/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { HTMLAttributes, useState } from 'react'
import { z } from 'zod'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { cn } from '@/lib/utils'
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/components/ui/form'
import { Input } from '@/components/ui/input'
import { PasswordInput } from '@/components/password-input'
import { useMutation } from '@tanstack/react-query'
import { login } from '@/api/access-tokens/api'
import { setAccessToken } from '@/stores/authStore'
import { toast } from '@/hooks/use-toast'
import { AxiosError } from 'axios'
import { ToastAction } from '@/components/ui/toast'
import { useLocation, useNavigate } from '@tanstack/react-router'
import { Button } from '@/components/button'

type UserAuthFormProps = HTMLAttributes<HTMLDivElement>

const formSchema = z.object({
  username: z
    .string(),
  password: z
    .string()
    .min(1, { message: 'Please enter your password' })
    .min(4, { message: 'Password must be at least 4 characters long' }),
});

export function UserAuthForm({ className, ...props }: UserAuthFormProps) {
  const [isLoading, setIsLoading] = useState(false)
  const navigate = useNavigate()

  const { search } = useLocation();
  const redirect = new URLSearchParams(search).get('redirect') || '/';

  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      username: 'root',
      password: '',
    },
  })

  const mutation = useMutation({
    mutationFn: (password: string) => login(password),
    retry: 0,
  });

  async function onSubmit(data: z.infer<typeof formSchema>) {
    setIsLoading(true)

    mutation.mutate(data.password, {
      onSuccess: (rootToken) => {
        setAccessToken(rootToken);
        setIsLoading(false);
        navigate({ to: redirect });
      },
      onError: (error) => {
        if (error instanceof AxiosError && error.response && error.response.status === 401) {
          toast({
            variant: "destructive",
            title: "Login Failed",
            description: "Invalid password. Please try again.",
            action: <ToastAction altText="Try again">Try again</ToastAction>,
          })
        } else {
          toast({
            variant: "destructive",
            title: "Something went wrong",
            description: (error as Error).message,
            action: <ToastAction altText="Try again">Try again</ToastAction>,
          })
        }
        setIsLoading(false)
      }
    });
  }

  return (
    <div className={cn('grid gap-6', className)} {...props}>
      <Form {...form}>
        <form onSubmit={form.handleSubmit(onSubmit)}>
          <div className='grid gap-2'>
            <FormField
              control={form.control}
              name='username'
              render={({ field }) => (
                <FormItem className='space-y-1'>
                  <FormLabel>Username</FormLabel>
                  <FormControl>
                    <Input disabled {...field} value={"root"} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              control={form.control}
              name='password'
              render={({ field }) => (
                <FormItem className='space-y-1'>
                  <div className='flex items-center justify-between'>
                    <FormLabel>Password</FormLabel>
                  </div>
                  <FormControl>
                    <PasswordInput placeholder='********' {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <Button className='mt-2' loading={isLoading}>
              Login
            </Button>
          </div>
        </form>
      </Form>
    </div>
  )
}