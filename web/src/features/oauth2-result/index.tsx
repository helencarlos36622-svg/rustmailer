/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { Card, CardHeader, CardContent } from "@/components/ui/card";
import { FixedHeader } from "@/components/layout/fixed-header";
import { Main } from "@/components/layout/main";
import { useLocation } from "@tanstack/react-router";
import { AlertCircle, CheckCircle2, Info } from "lucide-react";
import { Button } from "@/components/ui/button";

export default function OAuth2Result() {
    const { search } = useLocation();
    const params = new URLSearchParams(search);
    const error = params.get("error");
    const message = params.get("message");
    const success = params.get("success");

    return (
        <>
            <FixedHeader />
            <Main className="flex min-h-screen flex-col items-center justify-center p-4">
                <div className="w-full max-w-5xl">
                    {error ? (
                        <Card className="shadow-lg">
                            <CardHeader>
                                <h2 className="text-2xl font-semibold text-center ">
                                    OAuth2 Authentication Failed
                                </h2>
                            </CardHeader>
                            <CardContent className="space-y-4">
                                <div className="rounded-lg p-4">
                                    <div className="flex items-start">
                                        <AlertCircle className="h-12 w-12 flex-shrink-0 mt-0.5 text-red-500" />
                                        <div className="ml-3 flex-1">
                                            <h3 className="text-sm font-medium ">Error</h3>
                                            <div className="mt-2 text-sm bg-gray-100 dark:bg-gray-800 rounded p-4">
                                                <code className="whitespace-pre-wrap text-sm font-mono break-all rounded p-2">
                                                    {message?.replace(/\\n/g, "\n").replace(/\\"/g, "") || "An unknown error occurred."}
                                                </code>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                                <div className="flex justify-center gap-4 pt-2">
                                    <Button variant="outline" asChild>
                                        <a href="/oauth2">Try Again</a>
                                    </Button>
                                    <Button variant="link" asChild>
                                        <a href="/">Go Home</a>
                                    </Button>
                                </div>
                            </CardContent>
                        </Card>
                    ) : success ? (
                        <Card className="shadow-lg">
                            <CardHeader>
                                <h2 className="text-2xl font-semibold text-center ">
                                    OAuth2 Authentication Successful
                                </h2>
                            </CardHeader>
                            <CardContent className="space-y-4">
                                <div className="rounded-lg p-4">
                                    <div className="flex items-start">
                                        <CheckCircle2 className="h-12 w-12 flex-shrink-0 mt-0.5 text-green-500" />
                                        <div className="ml-3 flex-1">
                                            <h2 className="text-sm font-medium">Success</h2>
                                            <div className="mt-2 text-sm">
                                                Your email account has been successfully authenticated via OAuth2. The Access Token has been saved by the system and will be automatically updated as needed.
                                            </div>
                                        </div>
                                    </div>
                                </div>
                                <div className="flex justify-center pt-2">
                                    <Button asChild>
                                        <a href="/accounts">Go to Accounts</a>
                                    </Button>
                                </div>
                            </CardContent>
                        </Card>
                    ) : (
                        <Card className="shadow-lg">
                            <CardHeader>
                                <h2 className="text-2xl font-semibold text-center">
                                    OAuth2 Authentication Status
                                </h2>
                            </CardHeader>
                            <CardContent className="space-y-4">
                                <div className="rounded-lg p-4">
                                    <div className="flex items-start">
                                        <Info className="h-5 w-5 flex-shrink-0 mt-0.5" />
                                        <div className="ml-3 flex-1">
                                            <h3 className="text-sm font-medium">Information</h3>
                                            <div className="mt-2 text-sm">
                                                This page displays the result of your OAuth2 authentication. Currently, no specific status is available. Please try again if needed.
                                            </div>
                                        </div>
                                    </div>
                                </div>
                                <div className="flex justify-center pt-2">
                                    <Button variant="outline" asChild>
                                        <a href="/oauth2">Back to Login</a>
                                    </Button>
                                </div>
                            </CardContent>
                        </Card>
                    )}
                </div>
            </Main>
        </>
    );
}