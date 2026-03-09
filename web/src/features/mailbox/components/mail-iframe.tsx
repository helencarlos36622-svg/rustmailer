/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import React from 'react';

interface EmailIframeProps {
  emailHtml: string;
  height?: number;
}

const EmailIframe: React.FC<EmailIframeProps> = ({ emailHtml, height }) => {
  const encodedHtml = encodeURIComponent(emailHtml);
  const iframeSrc = `data:text/html;charset=utf-8,${encodedHtml}`;

  return (
    <iframe
      src={iframeSrc}
      sandbox="allow-scripts"
      className="w-full border-none"
      title="Email Content"
      style={{ height: height ?? '1800px' }}
    />
  );
};

export default EmailIframe;
