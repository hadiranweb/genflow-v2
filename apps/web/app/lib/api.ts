const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || "http://localhost:3000";

export class GenFlowApi {
  constructor(private readonly token?: string) {}

  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const response = await fetch(`${API_BASE_URL}${endpoint}`, {
      ...options,
      headers: {
        "Content-Type": "application/json",
        ...(this.token ? { Authorization: `Bearer ${this.token}` } : {}),
        ...options.headers,
      },
    });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return response.json() as Promise<T>;
  }

  health() {
    return this.request<{ status: string }>("/health");
  }

  generatePosition(data: unknown) {
    return this.request("/api/v2/positions/generate", {
      method: "POST",
      body: JSON.stringify(data),
    });
  }

  resolveMcp(data: unknown) {
    return this.request("/api/v2/mcp/resolve", {
      method: "POST",
      body: JSON.stringify(data),
    });
  }

  getPositions(params?: Record<string, string>) {
    const qs = params ? "?" + new URLSearchParams(params).toString() : "";
    return this.request<unknown[]>(`/api/v2/positions${qs}`);
  }

  getCandidates(params?: Record<string, string>) {
    const qs = params ? "?" + new URLSearchParams(params).toString() : "";
    return this.request<unknown[]>(`/api/v2/candidates${qs}`);
  }
}
