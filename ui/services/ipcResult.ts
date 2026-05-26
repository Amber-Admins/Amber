import type { IpcResult } from "../ipc.ts";

export class AppError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "AppError";
  }
}

export function toAppError(error: unknown): AppError {
  if (error instanceof AppError) {
    return error;
  }
  if (error instanceof Error) {
    return new AppError(error.message || "Unknown Error");
  }
  if (error && typeof error === "object") {
    if ("message" in error && error.message !== undefined && error.message !== null) {
      return new AppError(String((error as Record<string, unknown>).message));
    }
    if ("error" in error && error.error !== undefined && error.error !== null) {
      return new AppError(String((error as Record<string, unknown>).error));
    }
  }
  return new AppError(String(error));
}

export async function unwrapIpcResult<T>(promise: Promise<IpcResult<T>>): Promise<T> {
  const result = await promise;
  if ("ok" in result) {
    return result.ok;
  }
  throw new AppError(result.err);
}
