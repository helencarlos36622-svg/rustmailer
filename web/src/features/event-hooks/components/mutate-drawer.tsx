/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { useState } from 'react'
import { useQuery, useQueryClient, useMutation } from '@tanstack/react-query'
import { AxiosError } from 'axios'
import { toast } from '@/hooks/use-toast'
import { Button } from '@/components/ui/button'
import {
  Sheet,
  SheetClose,
  SheetContent,
  SheetDescription,
  SheetFooter,
  SheetHeader,
  SheetTitle,
} from '@/components/ui/sheet'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { Loader2, Terminal } from 'lucide-react'
import { Card, CardContent } from '@/components/ui/card'
import { ScrollArea } from '@/components/ui/scroll-area'
import { ToastAction } from '@/components/ui/toast'
import { zodResolver } from '@hookform/resolvers/zod'
import { EventHook } from '../data/schema'
import useMinimalAccountList from '@/hooks/use-minimal-account-list'
import { create_event_hook, event_examples, ResolveResult, update_event_hook, vrl_script_resolve } from '@/api/hook/api'
import { HttpForm, httpFormSchema, HttpEventHookForm } from './http-form'
import { NatsForm, natsFormSchema, NatsEventHookForm } from './nats-form'
import { useForm } from 'react-hook-form'

interface Props {
  open: boolean
  onOpenChange: (open: boolean) => void
  currentRow?: EventHook
}

const httpDefaultValues = {
  account_id: undefined,
  description: undefined,
  enabled: true,
  global: false,
  http: {
    target_url: '',
    http_method: 'Post',  // String, default to 'Post' method
    custom_headers: []  // Object, default to empty object
  },
  vrl_script: '',
  use_proxy: undefined,
  watched_events: []
};


const natsDefaultValues = {
  account_id: undefined,
  description: undefined,
  enabled: true,
  global: false,
  nats: {
    host: '',
    port: 4222,
    token: undefined,
    username: undefined,
    password: undefined,
    stream_name: '',
    namespace: ''
  },
  vrl_script: '',
  watched_events: []
};

export function EventHooksMutateDrawer({ open, onOpenChange, currentRow }: Props) {
  const [eventType, setEventType] = useState<'Nats' | 'Http' | undefined>(
    currentRow ? currentRow.hook_type : undefined
  )
  const queryClient = useQueryClient()
  const [inputJson, setInputJson] = useState<string | undefined>(undefined)
  const [resolveResult, setResolveResult] = useState<string | undefined>(undefined)

  const isUpdate = !!currentRow

  const { accountsOptions, isLoading } = useMinimalAccountList()

  const { data: eventExamples } = useQuery({
    queryKey: ['event-examples'],
    queryFn: event_examples,
    staleTime: 1000 * 60 * 30
  })

  const httpForm = useForm<HttpEventHookForm>({
    resolver: zodResolver(httpFormSchema),
    defaultValues: currentRow ? {
      account_id: currentRow.account_id ?? undefined,
      description: currentRow.description ?? undefined,
      enabled: currentRow.enabled,
      global: currentRow.global === 1,
      http: currentRow.http ? {
        ...currentRow.http,
        custom_headers: convertRecordToArray(currentRow.http.custom_headers)
      } : undefined,
      use_proxy: currentRow.use_proxy ?? undefined,
      vrl_script: currentRow.vrl_script,
      watched_events: currentRow.watched_events
    } : httpDefaultValues,
  })

  const natsForm = useForm<NatsEventHookForm>({
    resolver: zodResolver(natsFormSchema),
    defaultValues: currentRow ? {
      account_id: currentRow.account_id ?? undefined,
      description: currentRow.description,
      enabled: currentRow.enabled,
      global: currentRow.global === 1,
      nats: {
        ...currentRow?.nats,
        auth_type: currentRow?.nats?.auth_type || "None",
        token: currentRow?.nats?.token || undefined,
        username: currentRow?.nats?.username || undefined,
        password: currentRow?.nats?.password || undefined
      },
      vrl_script: currentRow.vrl_script,
      watched_events: currentRow.watched_events
    } : natsDefaultValues,
  })

  const createEventHookMutation = useMutation({
    mutationFn: (data: Record<string, any>) => create_event_hook(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['event-hook-list'] })
      toast({
        title: 'Event Hook Created Successfully',
        description: (
          <div>
            <p className="text-sm">Your event hook has been created.</p>
          </div>
        ),
      })
      onOpenChange(false)
      setEventType(undefined)
      httpForm.reset()
      natsForm.reset()
    },
    onError: handleCreateEventHookError
  })

  const updateEventHookMutation = useMutation({
    mutationFn: (data: Record<string, any>) => {
      return update_event_hook(currentRow!.id, data)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['event-hook-list'] })
      toast({
        title: 'Event Hook updated Successfully',
        description: (
          <div>
            <p className="text-sm">Your event hook has been created.</p>
          </div>
        ),
      })
      onOpenChange(false)
      setEventType(undefined)
      httpForm.reset()
      natsForm.reset()
    },
    onError: handleUpdateEventHookError
  })

  function handleCreateEventHookError(error: AxiosError) {
    const errorMessage = (error.response?.data as { message?: string })?.message ||
      error.message ||
      `failed to create event hook, please try again later`

    toast({
      variant: "destructive",
      title: `Create event hook failed`,
      description: errorMessage as string,
      action: <ToastAction altText="Try again">Try again</ToastAction>,
    })
    console.error(error)
  }

  function handleUpdateEventHookError(error: AxiosError) {
    const errorMessage = (error.response?.data as { message?: string })?.message ||
      error.message ||
      `failed to update event hook, please try again later`

    toast({
      variant: "destructive",
      title: `Update event hook failed`,
      description: errorMessage as string,
      action: <ToastAction altText="Try again">Try again</ToastAction>,
    })
    console.error(error)
  }

  const onHttpSubmit = (data: HttpEventHookForm) => {
    if (!data.global && !data.account_id) {
      httpForm.setError('account_id', {
        type: 'manual',
        message: 'Please select an account when not using global mode'
      });
      return;
    }

    const headersObject = data.http.custom_headers
      ? Object.fromEntries(
        data.http.custom_headers.map(({ key, value }) => [key, value])
      )
      : undefined
    const payload = {
      ...data,
      hook_type: 'Http',
      http: {
        ...data.http,
        custom_headers: headersObject,
      },
    }
    if (isUpdate) {
      updateEventHookMutation.mutate(payload)
    } else {
      createEventHookMutation.mutate(payload)
    }
  }

  const onNatsSubmit = (data: NatsEventHookForm) => {
    if (!data.global && !data.account_id) {
      natsForm.setError('account_id', {
        type: 'manual',
        message: 'Please select an account when not using global mode'
      });
      return;
    }

    const { auth_type } = data.nats
    if (auth_type === 'Token' && !data.nats.token) {
      natsForm.setError('nats.token', {
        type: 'manual',
        message: 'Token is required for token authentication'
      })
      return
    }
    if (auth_type === 'Password') {
      if (!data.nats.username) {
        natsForm.setError('nats.username', {
          type: 'manual',
          message: 'Username is required for password authentication'
        })
        return
      }
      if (!data.nats.password) {
        natsForm.setError('nats.password', {
          type: 'manual',
          message: 'Password is required for password authentication'
        })
        return
      }
    }
    const payload = {
      ...data,
      hook_type: 'Nats',
      nats: {
        ...data.nats,
        ...(auth_type === 'None'
          ? { token: undefined, username: undefined, password: undefined }
          : auth_type === 'Token'
            ? { username: undefined, password: undefined }
            : { token: undefined }
        ),
      }
    }
    if (isUpdate) {
      updateEventHookMutation.mutate(payload)
    } else {
      createEventHookMutation.mutate(payload)
    }
  }

  const runScriptTestMutation = useMutation({
    mutationFn: (data: Record<string, any>) => vrl_script_resolve(data),
    onSuccess: handleSuccess,
    onError: handleError
  })

  function handleSuccess(data: ResolveResult) {
    if (data.error) {
      setResolveResult(data.error)
    } else {
      setResolveResult(JSON.stringify(data.result, null, 2))
    }
  }

  function handleError(error: AxiosError) {
    const errorMessage = (error.response?.data as { message?: string })?.message ||
      error.message ||
      `failed to run script test, please try again later`

    toast({
      variant: "destructive",
      title: `Vrl script test failed`,
      description: errorMessage as string,
      action: <ToastAction altText="Try again">Try again</ToastAction>,
    })
    console.error(error)
  }

  const showErrorToast = (title: string) => {
    toast({
      variant: "destructive",
      title,
      action: <ToastAction altText="Try again">Try again</ToastAction>,
    })
  }

  const runTest = () => {
    if (!inputJson) {
      showErrorToast("Event Example is Empty")
      return
    }

    try {
      JSON.parse(inputJson)
    } catch (error) {
      showErrorToast("Input JSON is invalid. Please provide valid JSON data.")
      return
    }

    const form = eventType === "Http" ? httpForm : natsForm
    const formValues = form.getValues()
    const vrlScript = formValues.vrl_script

    if (!vrlScript?.trim()) {
      showErrorToast("No VRL script provided")
      return
    }

    const payload = {
      program: vrlScript,
      event: inputJson
    }

    runScriptTestMutation.mutate(payload)
  }

  return (
    <Sheet
      open={open}
      onOpenChange={(v) => {
        onOpenChange(v)
        httpForm.reset()
        natsForm.reset()
        setEventType(undefined)
      }}
    >
      <SheetContent className='md:w-[80rem]'>
        <SheetHeader className='text-left'>
          <SheetTitle>{isUpdate ? 'Update' : 'Create'} EventHook</SheetTitle>
          <SheetDescription>
            {isUpdate
              ? 'Update the event hook by providing necessary info. '
              : 'Add a new event hook by providing necessary info. '}
            Click save when you&apos;re done.
          </SheetDescription>
        </SheetHeader>
        {!eventType && <Alert>
          <Terminal className="h-4 w-4" />
          <AlertTitle>Heads up!</AlertTitle>
          <AlertDescription>
            <strong>EventHook Overview:</strong>
            <br />
            EventHook is a powerful tool that allows you to trigger actions or workflows based on specific events. Currently, EventHook supports the following approaches:
            <ul className="list-disc list-inside mt-2">
              <li>
                <strong>HTTP:</strong> Send and receive events via HTTP requests, enabling seamless integration with web services and APIs.
              </li>
              <li>
                <strong>Nats:</strong> Utilize Nats for high-performance, lightweight messaging between distributed systems. To learn more, refer to the official documentation:
                <a
                  href="https://docs.nats.io/"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-blue-600 hover:underline"
                >
                  Nats Documentation
                </a>.
              </li>
            </ul>
            With EventHook, you can easily configure event-driven workflows, ensuring real-time responsiveness and scalability for your applications.
          </AlertDescription>
        </Alert>}
        {!eventType && <div className="flex flex-col md:flex-row gap-4">
          <Card
            className="flex-1 h-24 border border-gray-200 dark:border-gray-700 rounded-lg p-4 hover:border-[hsla(48,96%,53%,1)] hover:bg-[hsla(48,96%,93%,1)] dark:hover:bg-[hsla(48,96%,20%,1)] hover:shadow-lg transition-all cursor-pointer flex items-center justify-center"
            onClick={() => {
              setEventType('Http')
            }}
          >
            <CardContent>
              <p className="text-gray-900 dark:text-gray-100">HTTP</p>
            </CardContent>
          </Card>
          <Card
            className="flex-1 h-24 border border-gray-200 dark:border-gray-700 rounded-lg p-4 hover:border-[hsla(48,96%,53%,1)] hover:bg-[hsla(48,96%,93%,1)] dark:hover:bg-[hsla(48,96%,20%,1)] hover:shadow-lg transition-all cursor-pointer flex items-center justify-center"
            onClick={() => {
              setEventType('Nats')
            }}
          >
            <CardContent>
              <p className="text-gray-900 dark:text-gray-100">Nats</p>
            </CardContent>
          </Card>
        </div>}
        {eventType && <ScrollArea className='h-full w-full pr-4 -mr-4 py-1'>
          {eventType === 'Http' &&
            <HttpForm
              form={httpForm}
              accountsOptions={accountsOptions}
              isLoading={isLoading}
              isUpdate={isUpdate}
              eventExamples={eventExamples}
              inputJson={inputJson}
              setInputJson={setInputJson}
              resolveResult={resolveResult}
              setResolveResult={setResolveResult}
              runTest={runTest}
            />
          }
          {eventType === 'Nats' &&
            <NatsForm
              form={natsForm}
              accountsOptions={accountsOptions}
              isLoading={isLoading}
              isUpdate={isUpdate}
              eventExamples={eventExamples}
              inputJson={inputJson}
              setInputJson={setInputJson}
              resolveResult={resolveResult}
              setResolveResult={setResolveResult}
              runTest={runTest}
            />
          }
        </ScrollArea>}

        {eventType && <SheetFooter className='gap-2'>
          <SheetClose asChild>
            <Button variant='outline'>Close</Button>
          </SheetClose>
          <Button
            form="eventhook-form"
            type="submit"
            onClick={eventType === 'Http'
              ? httpForm.handleSubmit(onHttpSubmit)
              : natsForm.handleSubmit(onNatsSubmit)}
            disabled={
              (isUpdate && (updateEventHookMutation.isPending || httpForm.formState.isSubmitting || natsForm.formState.isSubmitting)) ||
              (!isUpdate && (createEventHookMutation.isPending || httpForm.formState.isSubmitting || natsForm.formState.isSubmitting))
            }
            className="min-w-[100px] relative transition-all"
          >
            <span className="inline-flex items-center justify-center gap-2">
              {(isUpdate ? updateEventHookMutation.isPending : createEventHookMutation.isPending) && (
                <Loader2 className="h-4 w-4 animate-spin" />
              )}
              <span>
                {isUpdate
                  ? updateEventHookMutation.isPending
                    ? "Updating..."
                    : "Save changes"
                  : createEventHookMutation.isPending
                    ? "Creating..."
                    : "Save"}
              </span>
            </span>
          </Button>
        </SheetFooter>}
      </SheetContent>
    </Sheet>
  )
}

function convertRecordToArray(record: Record<string, string> | undefined) {
  if (!record) {
    return []
  }
  return Object.entries(record).map(([key, value]) => ({
    key,
    value,
  }))
}