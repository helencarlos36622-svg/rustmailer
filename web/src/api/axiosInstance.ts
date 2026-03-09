/*
 * Copyright © 2025-2026 rustmailer.com
 * Licensed under RustMailer License Agreement v1.0
 * Unauthorized use or distribution is prohibited.
 */

import { getAccessToken } from "@/stores/authStore";
import axios from "axios";

// Create an Axios instance
const baseURL = process.env.NODE_ENV === "production"
  ? "/" // Production: relative to the current domain
  : "http://localhost:15630"; // Development: Poem's backend server

const axiosInstance = axios.create({
  baseURL,
  timeout: 30000, // Timeout in milliseconds
  headers: {
    "Content-Type": "application/json",  // Explicitly setting Content-Type to application/json
  },
});

// Add a request interceptor to include the access token in headers
axiosInstance.interceptors.request.use(
  (config) => {
    const accessToken = getAccessToken(); // Retrieve access token from localStorage
    if (accessToken) {
      config.headers.Authorization = `Bearer ${accessToken}`;
    }
    return config;
  },
  (error) => {
    // Handle request errors
    return Promise.reject(error);
  }
);

// Add a response interceptor (optional)
axiosInstance.interceptors.response.use(
  (response) => response,
  (error) => {
    // Handle response errors
    //console.error("API error:", error.response?.data || error.message);
    return Promise.reject(error);
  }
);

export default axiosInstance;
