/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { Mail } from "./components/mail"
import { Main } from "@/components/layout/main"
import { FixedHeader } from "@/components/layout/fixed-header"

export default function Mailboxes() {
    const layout = localStorage.getItem("react-resizable-panels:layout:mail")
    const collapsed = localStorage.getItem("react-resizable-panels:collapsed")

    const defaultLayout = layout ? JSON.parse(layout) : undefined
    const defaultCollapsed = collapsed ? JSON.parse(collapsed) : undefined

    const lastSelectedAccountId = localStorage.getItem('mailbox:selectedAccountId') ?? undefined

    return (
        <>
            {/* ===== Top Heading ===== */}
            <FixedHeader />

            <Main higher>
                <Mail
                    defaultLayout={defaultLayout}
                    defaultCollapsed={defaultCollapsed}
                    lastSelectedAccountId={lastSelectedAccountId ? parseInt(lastSelectedAccountId) : undefined}
                    navCollapsedSize={2}
                />
            </Main>
        </>
    )
}