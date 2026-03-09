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
  DialogClose,
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
import { EmailTemplate } from '../data/schema'
import { useTheme } from '@/context/theme-context'
import AceEditor from '@/components/ace-editor'
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from '@/components/ui/accordion'
import { IconClipboardText } from '@tabler/icons-react'
import useMinimalAccountList from '@/hooks/use-minimal-account-list'
import { VirtualizedSelect } from '@/components/virtualized-select'
import { useMutation } from '@tanstack/react-query'
import { send_test_email } from '@/api/templates/api'
import { Loader2 } from 'lucide-react'

const isValidJson = (value: string) => {
  try {
    JSON.parse(value);
    return true;
  } catch (error) {
    return false;
  }
};

const sendTestFormSchema = z.object({
  account_id: z.number(),
  recipient: z
    .string()
    .min(1, { message: "Recipient is required." })
    .email({ message: "Please enter a valid email address." }),
  template_params: z
    .string()
    .optional()
    .transform((val) => val === "" ? undefined : val)
    .refine(
      (val) => val === undefined || isValidJson(val),
      {
        message: "Parameters must be a valid JSON string.",
      }
    ),
});

export type SendTestEmailForm = z.infer<typeof sendTestFormSchema>;

interface Props {
  currentRow: EmailTemplate
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function SentTestEmailDialog({ currentRow, open, onOpenChange }: Props) {
  const { theme } = useTheme()
  const form = useForm<SendTestEmailForm>({
    resolver: zodResolver(sendTestFormSchema),
    defaultValues: {
      account_id: currentRow.account ? currentRow.account.id : undefined,
      recipient: "",
      template_params: ""
    },
  });

  const { accountsOptions, isLoading } = useMinimalAccountList()

  const isPublic = !currentRow.account

  const mutation = useMutation({
    mutationFn: (values: SendTestEmailForm) => send_test_email(currentRow.id, values),
    onSuccess: () => {
      toast({
        title: "Test Email Sent",
        description: "The test email was successfully sent. Please check the recipient's inbox.",
      });
      onOpenChange(false);
    },
    onError: (error: any) => {
      toast({
        title: "Failed to Send Test Email",
        description: error.response?.data?.message || "An error occurred while sending the test email.",
        variant: "destructive",
      });
    },
  });

  const onSubmit = (values: SendTestEmailForm) => {
    mutation.mutate(values);
  }

  const formatJson = () => {
    try {
      let value = form.getValues('template_params');
      if (value) {
        let parsedValue = JSON.parse(value);
        let prettyJson = JSON.stringify(parsedValue, null, 2);
        form.setValue('template_params', prettyJson);
      } else {
        toast({
          title: 'No value to format',
          description: 'The "params" field is empty.',
        });
      }
    } catch (error) {
      toast({
        title: 'Formatting Error',
        description: 'The value is not a valid JSON string.',
        variant: 'destructive',
      });
    }
  };

  return (
    <Dialog
      open={open}
      onOpenChange={(state) => {
        form.reset()
        onOpenChange(state)
      }}
    >
      <DialogContent className='sm:max-w-7xl'>
        <DialogHeader className='text-left'>
          <DialogTitle>Send Test Email</DialogTitle>
          <DialogDescription>
            Send a test email using the selected template.
          </DialogDescription>
        </DialogHeader>
        <ScrollArea className='h-[42rem] w-full pr-4 -mr-4 py-1'>
          <Form {...form}>
            <form
              id='send-email-test-form'
              onSubmit={form.handleSubmit(onSubmit)}
              className='space-y-4 p-0.5'
            >
              <div className='space-y-6'>
                <Accordion type='multiple' defaultValue={["item-1", "item-2"]}>
                  <AccordionItem value="item-1">
                    <AccordionTrigger>
                      <div className="flex flex-col gap-y-1">
                        <span className="font-semibold">1.Email Configuration</span>
                        <span className="text-sm text-muted-foreground">
                          Set up your email account and recipient details.
                        </span>
                      </div>
                    </AccordionTrigger>
                    <AccordionContent>
                      <div className='space-y-2'>
                        <FormField
                          control={form.control}
                          name='account_id'
                          render={({ field }) => (
                            <FormItem className='flex flex-col gap-y-1 space-y-0'>
                              <FormLabel className='mb-1'>Account:</FormLabel>
                              <FormControl>
                                <VirtualizedSelect
                                  options={accountsOptions}
                                  className='w-full'
                                  isLoading={isLoading}
                                  onSelectOption={(values) => field.onChange(parseInt(values[0], 10))}
                                  defaultValue={`${field.value}`}
                                  placeholder="Select an account"
                                />
                              </FormControl>
                              {isPublic && <FormDescription>
                                Choose an email account (as SMTP server) for sending emails.
                              </FormDescription>}
                              <FormMessage />
                            </FormItem>
                          )}
                        />
                        <FormField
                          control={form.control}
                          name='recipient'
                          render={({ field }) => (
                            <FormItem className='flex flex-col gap-y-1 space-y-0'>
                              <FormLabel className='mb-1'>Recipient:</FormLabel>
                              <FormControl>
                                <Input
                                  placeholder='Please enter an email recipient address'
                                  {...field}
                                />
                              </FormControl>
                              <FormDescription>
                                After sending, check the recipient's inbox to verify the email content.
                              </FormDescription>
                              <FormMessage />
                            </FormItem>
                          )}
                        />
                      </div>
                    </AccordionContent>
                  </AccordionItem>
                  <AccordionItem value="item-2">
                    <AccordionTrigger>
                      <div className="flex flex-col gap-y-1">
                        <span className="font-semibold">2.Template Parameters</span>
                        <span className="text-sm text-muted-foreground">
                          Define dynamic parameters for your email template.
                        </span>
                      </div>
                    </AccordionTrigger>
                    <AccordionContent>
                      <FormField
                        control={form.control}
                        name='template_params'
                        render={({ field }) => (
                          <FormItem className='flex flex-col gap-y-1 space-y-0'>
                            <FormLabel className='mb-1'>Parameters:</FormLabel>
                            <FormControl>
                              <AceEditor
                                placeholder='Enter a JSON object for Handlebars rendering:
                                Template: <div>{{user.name}} ({{user.age}})</div>
                                Example input: {"user": {"name": "John", "age": 30}}'
                                value={field.value}
                                onChange={field.onChange}
                                className="h-[380px]"
                                mode='json'
                                theme={theme === "dark" ? 'monokai' : 'kuroir'}
                              />
                            </FormControl>
                            <FormDescription>
                              <Button
                                type="button"
                                variant={'default'}
                                className="px-2 py-1 text-xs space-x-1 mr-4 h-auto w-[120px]"
                                onClick={formatJson}
                              >
                                <span>Pretty Json</span><IconClipboardText size={18} />
                              </Button>
                              The parameters you provide will be used as variables in the Handlebars template, allowing dynamic content rendering in the email body.
                            </FormDescription>
                            <FormMessage />
                          </FormItem>
                        )}
                      />
                    </AccordionContent>
                  </AccordionItem>
                </Accordion>
              </div>
            </form>
          </Form>
        </ScrollArea>
        <DialogFooter>
          <DialogClose asChild>
            <Button variant='outline' className="px-2 py-1 text-sm h-auto">Close</Button>
          </DialogClose>
          <Button
            type='submit'
            form='send-email-test-form'
            disabled={mutation.isPending}
          >
            {mutation.isPending ? (
              <>
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                Sending...
              </>
            ) : (
              'Send'
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
