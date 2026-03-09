/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { CheckCircle } from "lucide-react"; // Using Lucide icon library
import { Button } from "@/components/ui/button"; // Using custom button component

export default function CompleteStep() {
    return (
        <div className="flex flex-col items-center justify-center h-[25rem] p-6">
            {/* Success Icon */}
            <div className="mb-6 text-green-500">
                <CheckCircle className="w-16 h-16" />
            </div>

            {/* Success Message */}
            <h1 className="text-3xl font-bold mb-4">Registration Successful!</h1>
            <p className="text-lg text-gray-600 mb-8 text-center">
                Your email account has been successfully added.
            </p>

            {/* Action Buttons */}
            <div className="mt-8 flex gap-4">
                <Button
                    variant="default"
                    onClick={() => {
                        // Redirect to home page
                        window.location.href = "/";
                    }}
                >
                    Authorize via OAuth2
                </Button>
                <Button
                    variant="outline"
                    onClick={() => {
                        // View details
                        window.location.href = "/accounts";
                    }}
                >
                    View Details
                </Button>
            </div>
        </div>
    );
}