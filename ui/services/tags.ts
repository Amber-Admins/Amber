import {
  nodeTagAdd,
  nodeTagRemove,
  nodeTagsGet,
  tagCreate,
  tagList,
  type Tag,
  type TagCreateInput,
} from "../ipc";
import { AppError, toAppError } from "./ipcResult.ts";

export type ServiceResult<T> = { data: T; error: null } | { data: null; error: AppError };

export async function listTags(): Promise<ServiceResult<Tag[]>> {
  const result = await tagList();
  if ("ok" in result) {
    return { data: result.ok, error: null };
  }
  return { data: null, error: toAppError(result.err) };
}

export async function createTag(input: TagCreateInput): Promise<ServiceResult<Tag>> {
  const result = await tagCreate(input);
  if ("ok" in result) {
    return { data: result.ok, error: null };
  }
  return { data: null, error: toAppError(result.err) };
}

export async function getNodeTags(nodeId: string): Promise<ServiceResult<Tag[]>> {
  const result = await nodeTagsGet(nodeId);
  if ("ok" in result) {
    return { data: result.ok, error: null };
  }
  return { data: null, error: toAppError(result.err) };
}

export async function addNodeTag(nodeId: string, tagId: string): Promise<ServiceResult<boolean>> {
  const result = await nodeTagAdd(nodeId, tagId);
  if ("ok" in result) {
    return { data: result.ok, error: null };
  }
  return { data: null, error: toAppError(result.err) };
}

export async function removeNodeTag(
  nodeId: string,
  tagId: string
): Promise<ServiceResult<boolean>> {
  const result = await nodeTagRemove(nodeId, tagId);
  if ("ok" in result) {
    return { data: result.ok, error: null };
  }
  return { data: null, error: toAppError(result.err) };
}
