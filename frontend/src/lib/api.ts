// API Client for LLMOps Control Center Rust Backend with Seamless Cloud/Vercel Fallback

import { handleCloudApiRequest } from "./cloudStore";

export const API_BASE_URL =
  process.env.NEXT_PUBLIC_API_URL || "http://localhost:8210";

function isRemoteWithoutCustomBackend(): boolean {
  if (process.env.NEXT_PUBLIC_API_URL) {
    return false;
  }
  if (typeof window !== "undefined") {
    const host = window.location.hostname;
    if (host !== "localhost" && host !== "127.0.0.1") {
      return true;
    }
  }
  return false;
}

export async function fetchApi<T>(endpoint: string, options?: RequestInit): Promise<T> {
  // When deployed on Vercel or remote host without a custom Rust API URL,
  // serve directly from the stateful Cloud/Edge store (persisted in sessionStorage)
  if (isRemoteWithoutCustomBackend()) {
    return handleCloudApiRequest(endpoint, options) as T;
  }

  const url = endpoint.startsWith("http")
    ? endpoint
    : `${API_BASE_URL}/api/v1${endpoint.startsWith("/") ? endpoint : `/${endpoint}`}`;

  try {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), 3500);

    const res = await fetch(url, {
      ...options,
      headers: {
        "Content-Type": "application/json",
        ...options?.headers,
      },
      cache: "no-store",
      signal: controller.signal,
    });

    clearTimeout(timeoutId);

    if (!res.ok) {
      const errorText = await res.text();
      let errorMessage = `HTTP Error ${res.status}`;
      try {
        const errorJson = JSON.parse(errorText);
        if (errorJson.error?.message) {
          errorMessage = errorJson.error.message;
        }
      } catch {
        if (errorText) errorMessage = errorText;
      }
      throw new Error(errorMessage);
    }

    return await res.json();
  } catch (err: any) {
    // Graceful fallback to stateful Cloud Engine if local Rust backend is not running
    console.warn(`Rust API unreachable on ${endpoint}, using embedded Cloud Fallback Store:`, err?.message || err);
    return handleCloudApiRequest(endpoint, options) as T;
  }
}
