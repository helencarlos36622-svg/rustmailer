/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import ContentSection from '../components/content-section'
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { IconCheck, IconCopy } from '@tabler/icons-react'
import { useCallback, useState } from 'react'
import { ConfirmDialog } from '@/components/confirm-dialog'
import { useMutation } from '@tanstack/react-query'
import { reset_root_token, reset_root_password } from '@/api/access-tokens/api'
import { toast } from '@/hooks/use-toast'
import { ToastAction } from '@/components/ui/toast'
import { PasswordInput } from '@/components/password-input'
import { resetAccessToken, setAccessToken } from '@/stores/authStore'
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert'
import { BellRing } from 'lucide-react'
import { useNavigate } from '@tanstack/react-router'

const useResetRootToken = () =>
  useMutation({ mutationFn: reset_root_token, retry: 0 });

const useResetRootPassword = () =>
  useMutation({
    mutationFn: (password: string) => reset_root_password(password),
    retry: 0,
  });

export default function RootAccess() {
  const navigate = useNavigate()

  const [openToken, setOpenToken] = useState(false);
  const [openPassword, setOpenPassword] = useState(false);

  const [newToken, setNewToken] = useState<string | null>(null);
  const [isCopied, setIsCopied] = useState(false);
  const [newPassword, setNewPassword] = useState("");

  const tokenMutation = useResetRootToken();
  const passwordMutation = useResetRootPassword();

  const onConfirmToken = useCallback(() => {
    tokenMutation.mutate(undefined, {
      onSuccess: (data) => {
        setNewToken(data);
        setAccessToken(data);
        toast({
          title: "The root token has been reset",
          description: "Your login information has been updated. No need to log in again.",
          action: <ToastAction altText="Close">Close</ToastAction>,
        });
        setOpenToken(false);
      },
    });
  }, [tokenMutation]);

  const onConfirmPassword = useCallback(() => {
    if (!newPassword || newPassword.length < 6) {
      toast({
        variant: "destructive",
        title: "Invalid password",
        description: "The root password must be at least 6 characters long.",
        action: <ToastAction altText="Close">Close</ToastAction>,
      });
      return;
    }
    passwordMutation.mutate(newPassword, {
      onSuccess: () => {
        toast({
          title: "The root password has been reset",
          description: "Use the new password for your next login.",
          action: <ToastAction altText="Close">Close</ToastAction>,
        });
        setNewPassword("");
        setOpenPassword(false);
        resetAccessToken()
        navigate({ to: '/sign-in' })
      },
    });
  }, [newPassword, passwordMutation]);

  const onCopy = useCallback(async () => {
    if (newToken) {
      try {
        await navigator.clipboard.writeText(newToken);
        setIsCopied(true);
      } catch (err) {
        toast({
          variant: "destructive",
          title: "Failed to copy text",
          description: (err as Error).message,
          action: <ToastAction altText="Try again">Try again</ToastAction>,
        });
      }
    }
  }, [newToken]);

  return (
    <ContentSection
      title="Root"
      desc="Manage root access credentials including token and password."
      showHeader={false}
    >
      <div>
        {/* Reset Root Token */}
        <Card className="flex flex-col mb-6">
          <CardHeader>
            <CardTitle>Reset Root Token</CardTitle>
            <CardDescription>
              Invalidate the old root token and generate a new one.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <Alert>
              <BellRing />
              <AlertTitle>Warning:</AlertTitle>
              <AlertDescription>
                <p className="text-sm text-warning-dark">
                  Resetting the root token will invalidate the old one immediately.
                </p>
              </AlertDescription>
            </Alert>
          </CardContent>
          <CardFooter>
            <Button className="ml-auto" onClick={() => setOpenToken(true)}>
              Reset Token
            </Button>
            <ConfirmDialog
              key="root-token-reset"
              destructive
              open={openToken}
              onOpenChange={setOpenToken}
              handleConfirm={onConfirmToken}
              className="max-w-xl"
              title="Reset Root Token"
              desc={
                <>
                  <Alert>
                    <BellRing />
                    <AlertTitle>Important:</AlertTitle>
                    <AlertDescription>
                      <p className="text-sm text-warning-dark">
                        You are about to reset the root access token. This action cannot be undone.
                      </p>
                    </AlertDescription>
                  </Alert>
                  {newToken && (
                    <div className="flex w-full mt-8 mb-8 items-center space-x-2">
                      <PasswordInput value={newToken} className="w-full" readOnly />
                      <Button onClick={onCopy}>
                        {isCopied ? (
                          <IconCheck className="h-5 w-5" aria-hidden="true" />
                        ) : (
                          <IconCopy className="h-5 w-5" aria-hidden="true" />
                        )}
                      </Button>
                    </div>
                  )}
                </>
              }
              confirmText="Reset"
              isLoading={tokenMutation.isPending}
            />
          </CardFooter>
        </Card>
        {/* Reset Root Password */}
        <Card className="flex flex-col">
          <CardHeader>
            <CardTitle>Reset Root Password</CardTitle>
            <CardDescription>
              Change the root password. You will need the new password for future logins.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <Alert>
              <BellRing />
              <AlertTitle>Caution:</AlertTitle>
              <AlertDescription>
                <p className="text-sm text-warning-dark">
                  Resetting the root password requires you to remember the new one. The old password will no longer work.
                </p>
              </AlertDescription>
            </Alert>
          </CardContent>
          <CardFooter>
            <Button className="ml-auto" onClick={() => setOpenPassword(true)}>
              Reset Password
            </Button>
            <ConfirmDialog
              key="root-password-reset"
              destructive
              open={openPassword}
              onOpenChange={setOpenPassword}
              handleConfirm={onConfirmPassword}
              className="max-w-xl"
              title="Reset Root Password"
              desc={
                <div className="flex flex-col gap-4">
                  <Alert>
                    <BellRing />
                    <AlertTitle>Important:</AlertTitle>
                    <AlertDescription>
                      <p className="text-sm text-warning-dark">
                        You are about to reset the root password. Make sure to store it securely.
                      </p>
                    </AlertDescription>
                  </Alert>
                  <PasswordInput
                    value={newPassword}
                    onChange={(e) => setNewPassword(e.target.value)}
                    placeholder="Enter new root password"
                    className="w-full"
                  />
                </div>
              }
              confirmText="Reset"
              isLoading={passwordMutation.isPending}
            />
          </CardFooter>
        </Card>
      </div>
    </ContentSection>
  );
}
