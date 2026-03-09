/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { z } from 'zod'

export const eventTypeSchema = z.enum([
  "AccountFirstSyncCompleted",
  "EmailAddedToFolder",
  "EmailBounce",
  "EmailFeedBackReport",
  "EmailFlagsChanged",
  "EmailSendingError",
  "EmailSentSuccess",
  "MailboxCreation",
  "MailboxDeletion",
  "UIDValidityChange",
  "EmailOpened",
  "EmailLinkClicked"
]);

export const eventTypeDescriptions: Record<z.infer<typeof eventTypeSchema>, string> = {
  AccountFirstSyncCompleted: "Fired after an account's initial sync finishes successfully",
  EmailAddedToFolder: "Triggered when message appears in folder (delivery/move/copy/append)",
  EmailBounce: "Indicates an email failed to deliver (bounced)",
  EmailFeedBackReport: "Triggered when a spam/abuse report is received for an email",
  EmailFlagsChanged: "Fired when email flags (read, starred, etc) are modified",
  EmailSendingError: "Triggered on EACH failed sending attempt (including retries)",
  EmailSentSuccess: "Confirmed SMTP delivery success (SMTP server or MTA accepted the message)",
  MailboxCreation: "Triggered when a new mailbox is created for an account",
  MailboxDeletion: "Fired when a mailbox is permanently removed",
  UIDValidityChange: "Advanced: Occurs when a mailbox's UID validity changes",
  EmailOpened: "Represents an event triggered when an email is opened by a recipient.",
  EmailLinkClicked: "Represents an event triggered when a link in an email is clicked by a recipient."
};

export const eventTypeOptions = eventTypeSchema.options.map((eventType) => ({
  value: eventType,
  label: eventType,
  description: eventTypeDescriptions[eventType]
}));
