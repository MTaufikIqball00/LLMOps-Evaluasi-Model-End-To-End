// API Client for LLMOps Control Center Rust Backend with Transparent Cloud/Vercel Fallback

import { handleCloudApiRequest } from "./cloudStore";

export const API_BASE_URL =
  process.env.NEXT_PUBLIC_API_URL || "http://localhost:8210";

export type ApiConnectionMode = "checking" | "live" | "cloud_fallback";

let currentConnectionMode: ApiConnectionMode = "checking";
const modeListeners = new Set<(mode: ApiConnectionMode) => void>();

function setConnectionMode(mode: ApiConnectionMode) {
  if (currentConnectionMode !== mode) {
    currentConnectionMode = mode;
    modeListeners.forEach((cb) => cb(mode));
  }
}

export function getConnectionMode(): ApiConnectionMode {
  return currentConnectionMode;
}

export function subscribeConnectionMode(
  callback: (mode: ApiConnectionMode) => void
): () => void {
  modeListeners.add(callback);
  callback(currentConnectionMode);
  return () => {
    modeListeners.delete(callback);
  };
}

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
    setConnectionMode("cloud_fallback");
    return handleCloudApiRequest(endpoint, options) as T;
  }

  const url = endpoint.startsWith("http")
    ? endpoint
    : `${API_BASE_URL}/api/v1${endpoint.startsWith("/") ? endpoint : `/${endpoint}`}`;

  const adminApiKey = process.env.NEXT_PUBLIC_ADMIN_API_KEY;
  const authHeaders: Record<string, string> = adminApiKey
    ? { "X-API-Key": adminApiKey }
    : {};

  try {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), 3500);

    const res = await fetch(url, {
      ...options,
      headers: {
        "Content-Type": "application/json",
        ...authHeaders,
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
      setConnectionMode("live");
      throw new Error(errorMessage);
    }

    setConnectionMode("live");
    return await res.json();
  } catch (err: any) {
    // Only fall back on transport/abort/unreachable errors, not validation errors returned by live backend
    if (currentConnectionMode === "live" && err?.message && !err.name?.includes("Abort") && !String(err.message).includes("Failed to fetch")) {
      throw err;
    }
    setConnectionMode("cloud_fallback");
    console.warn(
      `Rust API unreachable on ${endpoint}, using embedded Cloud Fallback Store:`,
      err?.message || err
    );
    return handleCloudApiRequest(endpoint, options) as T;
  }
}
