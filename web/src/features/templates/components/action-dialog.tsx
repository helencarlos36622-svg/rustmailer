/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { z } from 'zod'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { toast } from '@/hooks/use-toast'
import Handlebars from 'handlebars';
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
import { ScrollArea } from '@/components/ui/scroll-area'
import { Textarea } from '@/components/ui/textarea'
import { EmailTemplate } from '../data/schema'
import { useTheme } from '@/context/theme-context'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { Loader2, Terminal } from 'lucide-react'
import useMinimalAccountList from '@/hooks/use-minimal-account-list';
import { VirtualizedSelect } from '@/components/virtualized-select';
import { Switch } from '@/components/ui/switch';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { useState } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { create_template, update_template } from '@/api/templates/api';
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from '@/components/ui/accordion';
import { CodeEditorWithDraft } from '@/components/editor-with-draft';

const emailTemplateFormSchema = z.object({
  public: z.optional(z.boolean()),
  account_id: z.optional(z.number()),
  description: z.optional(
    z.string().max(255, { message: "Description must not exceed 255 characters." })
  ),
  subject: z.string({
    required_error: 'Please enter your subject.',
  }).min(1, { message: "Subject is required." }).transform((value, ctx) => {
    try {
      Handlebars.parse(value);
    } catch (error) {
      ctx.addIssue({
        code: z.ZodIssueCode.custom,
        message: (error as Error).message,
      });
      return z.NEVER;
    }
    return value
  }),
  preview: z.optional(z.string().max(255, { message: "Preview must not exceed 255 characters." }).nullable() // 允许 null
    .transform((val) => (val === null ? undefined : val)).transform((value, ctx) => {
      if (value !== undefined) {
        try {
          Handlebars.parse(value);
        } catch (error) {
          ctx.addIssue({
            code: z.ZodIssueCode.custom,
            message: (error as Error).message,
          });
          return z.NEVER;
        }
      }
      return value;
    })),
  format: z.string().optional(),
  text: z.string().optional(),
  html: z.optional(
    z.string()
      .min(1, { message: "HTML content cannot be empty if provided." })
      .transform((value, ctx) => {
        try {
          Handlebars.parse(value);
        } catch (error) {
          ctx.addIssue({
            code: z.ZodIssueCode.custom,
            message: (error as Error).message,
          });
          return z.NEVER;
        }
        return value;
      })
  )
}).superRefine((data, ctx) => {
  if (!data.public && !data.account_id) {
    ctx.addIssue({
      code: z.ZodIssueCode.custom,
      message: 'Account is required when "Public" is not selected.',
      path: ['account'],
    });
  }
  if (data.html && !data.format) {
    ctx.addIssue({
      code: z.ZodIssueCode.custom,
      message: 'Format is required when "Html" is set.',
      path: ['format'],
    });
  }
});

export type EmailTemplateForm = z.infer<typeof emailTemplateFormSchema>;

interface Props {
  currentRow?: EmailTemplate
  open: boolean
  onOpenChange: (open: boolean) => void
}

const defaultValues = {
  account_id: undefined,
  public: false,
  description: undefined,
  subject: '',
  format: undefined,
  preview: undefined,
  html: undefined,
  text: undefined
};


const mapCurrentRowToFormValues = (currentRow: EmailTemplate) => {
  let data = {
    account_id: currentRow.account?.id,
    public: true,
    description: currentRow.description,
    preview: currentRow.preview,
    subject: currentRow.subject,
    text: currentRow.text,
    html: currentRow.html,
    format: currentRow.format
  };

  if (currentRow.account) {
    data.public = false
  }
  return data;
};

export function TemplateActionDialog({ currentRow, open, onOpenChange }: Props) {
  const queryClient = useQueryClient();
  const { theme } = useTheme()
  const isEdit = !!currentRow
  const form = useForm<EmailTemplateForm>({
    resolver: zodResolver(emailTemplateFormSchema),
    defaultValues: isEdit
      ? mapCurrentRowToFormValues(currentRow)
      : defaultValues,
  });
  const { accountsOptions, isLoading } = useMinimalAccountList();

  const [showFormatMode, setShowFormatMode] = useState(true);
  const selectedFormat = form.watch('format');

  const createTemplateMutation = useMutation({
    mutationFn: create_template,
    retry: 0,
    onSuccess: () => {
      toast({
        title: 'Template created successfully',
        description: 'The email template has been created.',
      });
      form.reset();
      queryClient.invalidateQueries({ queryKey: ['email-templates-list'] });
      onOpenChange(false);
    },
    onError: (error: Error) => {
      toast({
        title: 'Failed to create template',
        description: `${error.message}`,
        variant: 'destructive',
      });
    },
  });


  const updateTemplateMutation = useMutation({
    mutationFn: (payload: EmailTemplateForm) => update_template(currentRow?.id!, payload),
    retry: 0,
    onSuccess: () => {
      toast({
        title: 'Template updated successfully',
        description: 'The email template has been updated.',
      });
      form.reset();
      queryClient.invalidateQueries({ queryKey: ['email-templates-list'] });
      onOpenChange(false);
    },
    onError: (error: Error) => {
      toast({
        title: 'Failed to update template',
        description: `${error.message}`,
        variant: 'destructive',
      });
    },
  });

  const onSubmit = (values: EmailTemplateForm) => {
    if (isEdit) {
      updateTemplateMutation.mutate(values);
    } else {
      createTemplateMutation.mutate(values);
    }
  }

  const getEditorMode = () => {
    if (!showFormatMode) return 'handlebars';
    return (selectedFormat ?? 'html').toLowerCase(); // 'html' or 'markdown'
  };

  return (
    <Dialog
      open={open}
      onOpenChange={(state) => {
        form.reset()
        onOpenChange(state)
      }}
    >
      <DialogContent className='max-w-7xl'>
        <DialogHeader className='text-left mb-4'>
          <DialogTitle>{isEdit ? 'Edit Template' : 'Add New Template'}</DialogTitle>
          <DialogDescription>
            {isEdit ? 'Update the email template here. ' : 'Create new email template here. '}
            Click save when you&apos;re done.
          </DialogDescription>
        </DialogHeader>
        <ScrollArea className='h-[42rem] w-full pr-4 -mr-4 py-1'>
          <Form {...form}>
            <form
              id='template-form'
              onSubmit={form.handleSubmit(onSubmit)}
              className='space-y-4 p-0.5'
            >
              <div className='space-y-6'>
                <FormField
                  control={form.control}
                  name='public'
                  render={({ field }) => (
                    <FormItem className='flex flex-row items-center justify-between rounded-lg border p-4'>
                      <div className='space-y-0.5'>
                        <FormLabel className='text-base'>
                          Public Template
                        </FormLabel>
                        <FormDescription>
                          This template will be publicly available to all accounts.
                        </FormDescription>
                      </div>
                      <FormControl>
                        <Switch
                          checked={field.value}
                          onCheckedChange={(checked) => {
                            field.onChange(checked);
                            if (checked) {
                              form.setValue('account_id', undefined);
                            }
                          }}
                        />
                      </FormControl>
                    </FormItem>
                  )}
                />
                <div className="flex space-x-4">
                  <div className="flex-1">
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
                          <FormDescription>
                            Select the account to which this email template belongs.
                          </FormDescription>
                          <FormMessage />
                        </FormItem>
                      )}
                    />
                  </div>
                </div>
                <Alert>
                  <Terminal className="h-4 w-4" />
                  <AlertTitle>Heads up!</AlertTitle>
                  <AlertDescription>
                    You <strong>must</strong> use <a
                      href="https://handlebarsjs.com/guide/"
                      target="_blank"
                      rel="noopener noreferrer"
                      className="text-yellow-700 hover:underline"
                    >
                      <strong>Handlebars</strong>
                    </a> to write your templates. Handlebars is the required templating language for this form, enabling dynamic content generation with variables, conditions, loops, and more.
                    <br />
                    To learn how to write templates using Handlebars, refer to the official documentation:
                    <a
                      href="https://handlebarsjs.com/guide/"
                      target="_blank"
                      rel="noopener noreferrer"
                      className="text-yellow-700 hover:underline"
                    >
                      Handlebars.js Guide
                    </a>
                    <br />
                    Ensure your templates comply with Handlebars syntax to avoid errors and ensure proper functionality.
                  </AlertDescription>
                </Alert>
                <FormField
                  control={form.control}
                  name='subject'
                  render={({ field }) => (
                    <FormItem className='flex flex-col gap-y-1 space-y-0'>
                      <FormLabel className='mb-1'>Subject:</FormLabel>
                      <FormControl>
                        <CodeEditorWithDraft
                          value={field.value}
                          onChange={field.onChange}
                          localStorageKey="draft_template_subject"
                          mode="handlebars"
                          placeholder='Enter your email subject tempalte here.'
                          theme={theme === "dark" ? 'monokai' : 'kuroir'}
                          className="h-[40px]"
                        />
                      </FormControl>
                      <FormDescription>
                        This is the template input for the email subject line. Use Handlebars syntax for dynamic content.
                      </FormDescription>
                      <FormMessage />
                    </FormItem>
                  )}
                />
                <FormField
                  control={form.control}
                  name='preview'
                  render={({ field }) => (
                    <FormItem className='flex flex-col gap-y-1 space-y-0'>
                      <FormLabel className='mb-1'>Preview:</FormLabel>
                      <FormControl>
                        <CodeEditorWithDraft
                          value={field.value}
                          onChange={field.onChange}
                          localStorageKey="draft_template_preview"
                          mode="handlebars"
                          placeholder='Enter your email preview tempalte here.'
                          theme={theme === "dark" ? 'monokai' : 'kuroir'}
                          className="h-[120px]"
                        />
                      </FormControl>
                      <FormDescription>(Optional)</FormDescription>
                      <FormDescription>
                        This is the template input for the email preview text (optional). Use Handlebars syntax for dynamic content.
                      </FormDescription>
                      <FormMessage />
                    </FormItem>
                  )}
                />
                <Accordion type="single" collapsible defaultValue="item-1">
                  <AccordionItem value="item-1">
                    <AccordionTrigger>Plain Text Content</AccordionTrigger>
                    <AccordionContent>
                      <FormField
                        control={form.control}
                        name='text'
                        render={({ field }) => (
                          <FormItem className='flex flex-col gap-y-1 space-y-0'>
                            <FormControl>
                              <CodeEditorWithDraft
                                value={field.value}
                                onChange={field.onChange}
                                localStorageKey="draft_template_text"
                                mode="handlebars"
                                placeholder='Enter your email text content tempalte here.'
                                theme={theme === "dark" ? 'monokai' : 'kuroir'}
                                className="h-[40rem]"
                              />
                            </FormControl>
                            <FormDescription>(Optional) Plain text content of the email.</FormDescription>
                            <FormMessage />
                          </FormItem>
                        )}
                      />
                    </AccordionContent>
                  </AccordionItem>
                </Accordion>
                <Accordion type="single" collapsible defaultValue="item-1">
                  <AccordionItem value="item-1">
                    <AccordionTrigger>HTML/Markdown version with formatting</AccordionTrigger>
                    <AccordionContent>
                      <FormField
                        control={form.control}
                        name="format"
                        render={({ field }) => (
                          <FormItem>
                            <FormLabel>Format</FormLabel>
                            <Select onValueChange={field.onChange} defaultValue={field.value}>
                              <FormControl>
                                <SelectTrigger>
                                  <SelectValue placeholder="Select a content format" />
                                </SelectTrigger>
                              </FormControl>
                              <SelectContent>
                                <SelectItem value="Html">Html</SelectItem>
                                <SelectItem value="Markdown">Markdown</SelectItem>
                              </SelectContent>
                            </Select>
                            <FormDescription>
                              Choose the format for the email template content (e.g., HTML or Markdown).
                            </FormDescription>
                            <FormMessage />
                          </FormItem>
                        )}
                      />
                      <FormField
                        control={form.control}
                        name='html'
                        render={({ field }) => (
                          <FormItem className='flex flex-col gap-y-1 space-y-0'>
                            <div className='flex items-center justify-between mb-1'>
                              <FormLabel>Message:</FormLabel>
                              <div className='flex items-center space-x-2'>
                                <span className='text-sm'>
                                  {showFormatMode ? selectedFormat : 'Handlebars'}
                                </span>
                                <Switch
                                  checked={showFormatMode}
                                  onCheckedChange={setShowFormatMode}
                                />
                              </div>
                            </div>
                            <FormControl>
                              <CodeEditorWithDraft
                                value={field.value}
                                onChange={field.onChange}
                                localStorageKey="draft_template_html"
                                mode={getEditorMode() as 'handlebars' | 'json' | 'markdown'}
                                placeholder='Enter your email html content tempalte here.'
                                theme={theme === "dark" ? 'monokai' : 'kuroir'}
                                className="h-[1000px]"
                              />
                            </FormControl>
                            <FormDescription>
                              This is the template input for the email body. Use Handlebars syntax for dynamic content. The final content will be sent as HTML.
                            </FormDescription>
                            <FormMessage />
                          </FormItem>
                        )}
                      />
                    </AccordionContent>
                  </AccordionItem>
                </Accordion>
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
                          className="max-h-[180px] min-h-[140px]"
                        />
                      </FormControl>
                      <FormDescription>(Optional)</FormDescription>
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
            type='submit'
            form='template-form'
            disabled={isEdit ? updateTemplateMutation.isPending : createTemplateMutation.isPending}
          >
            {isEdit ? (
              updateTemplateMutation.isPending ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Saving...
                </>
              ) : (
                "Save changes"
              )
            ) : (
              createTemplateMutation.isPending ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Creating...
                </>
              ) : (
                "Creat"
              )
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
