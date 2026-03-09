/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import React from "react";
import { GitHubLogoIcon } from "@radix-ui/react-icons";

interface GithubLinkButtonProps {
  /** GitHub repository or profile URL */
  href?: string;
  /** Icon size (default: 20) */
  size?: number;
  /** Optional tooltip title */
  title?: string;
}

export const GithubLinkButton: React.FC<GithubLinkButtonProps> = ({
  href = "https://github.com/rustmailer/rustmailer",
  size = 20,
  title = "View on GitHub",
}) => {
  return (
    <a
      href={href}
      target="_blank"
      rel="noopener noreferrer"
      title={title}
      className="inline-flex items-center justify-center rounded-full p-2 text-muted-foreground hover:text-foreground hover:bg-muted transition-colors"
    >
      <GitHubLogoIcon className="w-5 h-5" style={{ width: size, height: size }} />
    </a>
  );
};
