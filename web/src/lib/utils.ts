/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { type ClassValue, clsx } from 'clsx'
import { twMerge } from 'tailwind-merge'

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}


export const formatFileSize = (sizeInBytes: number): string => {
  if (sizeInBytes < 1024) {
    return `${sizeInBytes} B`;
  } else if (sizeInBytes < 1024 * 1024) {
    return `${(sizeInBytes / 1024).toFixed(2)} KB`;
  } else {
    return `${(sizeInBytes / (1024 * 1024)).toFixed(2)} MB`;
  }
};



export const validateFlag = (input: string): string | null => {
  // Check if the string is empty
  if (input.length === 0) {
    return `'input' cannot be empty.`;
  }

  // Check if the length is greater than 64 characters
  if (input.length > 64) {
    return `'input' cannot be longer than 64 characters.`;
  }

  // Check if the string starts with a letter and contains only letters, numbers, underscores, or dashes
  const regex = /^[a-zA-Z][a-zA-Z0-9_-]*$/;
  if (!regex.test(input)) {
    return `'input' must start with a letter and can only contain letters, numbers, underscores, or dashes.`;
  }

  // If all checks pass, return null
  return null;
};

