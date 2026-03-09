/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

const ACCESS_TOKEN_KEY = 'f4d3e92d7b1241a8a0b2e7cdb5c5d19d';

export const setAccessToken = (token: string) => {
  const data = {
    accessToken: token,
    expiresAt: Date.now() + 5 * 24 * 60 * 60 * 1000, // 5 days in milliseconds
  };
  localStorage.setItem(ACCESS_TOKEN_KEY, JSON.stringify(data));
};

export const getAccessToken = (): string => {
  const item = localStorage.getItem(ACCESS_TOKEN_KEY);
  if (item) {
    try {
      const data = JSON.parse(item);
      if (Date.now() < data.expiresAt) {
        return data.accessToken;
      } else {
        resetAccessToken(); // Clear the expired token
      }
    } catch (error) {
      console.error('Error parsing access token:', error);
      resetAccessToken(); // Clear invalid data
    }
  }
  return '';
};

export const resetAccessToken = () => {
  localStorage.removeItem(ACCESS_TOKEN_KEY);
};