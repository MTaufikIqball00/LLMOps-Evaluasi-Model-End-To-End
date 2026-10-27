// API Client for LLMOps Control Center Rust Backend

export const API_BASE_URL =
  process.env.NEXT_PUBLIC_API_URL || "http://localhost:8210";

export async function fetchApi<T>(endpoint: string, options?: RequestInit): Promise<T> {
  const url = endpoint.startsWith("http")
    ? endpoint
    : `${API_BASE_URL}/api/v1${endpoint.startsWith("/") ? endpoint : `/${endpoint}`}`;

  try {
    const res = await fetch(url, {
      ...options,
      headers: {
        "Content-Type": "application/json",
        ...options?.headers,
      },
      cache: "no-store",
    });

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
    console.error(`API Fetch Error on ${endpoint}:`, err);
    throw err;
  }
}
