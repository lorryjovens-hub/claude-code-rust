import { useMutation, useQuery, UseMutationResult, UseQueryResult } from "@tanstack/react-query";
import { chatApi, taskApi, modelApi, settingsApi, systemApi } from "@/lib/api";
import type { Conversation, Message, Task, Model, ModelProvider, AppSettings } from "@/lib/api";

export function useConversations(): UseQueryResult<Conversation[]> {
  return useQuery({ queryKey: ["conversations"], queryFn: chatApi.getConversations });
}

export function useConversation(id: string): UseQueryResult<Conversation | null> {
  return useQuery({ queryKey: ["conversation", id], queryFn: () => chatApi.getConversation(id) });
}

export function useCreateConversation(): UseMutationResult<Conversation, Error, { title?: string; model?: string }> {
  return useMutation<Conversation, Error, { title?: string; model?: string }>({
    mutationFn: ({ title, model }) => chatApi.createConversation(title, model)
  });
}

export function useSendMessage(): UseMutationResult<Message, Error, { message: string; conversationId?: string; model?: string }> {
  return useMutation<Message, Error, { message: string; conversationId?: string; model?: string }>({
    mutationFn: ({ message, conversationId, model }) => chatApi.sendMessage(message, conversationId, model)
  });
}

export function useDeleteConversation(): UseMutationResult<void, Error, string> {
  return useMutation({ mutationFn: chatApi.deleteConversation });
}

export function useClearConversation(): UseMutationResult<void, Error, string> {
  return useMutation({ mutationFn: chatApi.clearConversation });
}

export function useTasks(): UseQueryResult<Task[]> {
  return useQuery({ queryKey: ["tasks"], queryFn: taskApi.getTasks });
}

export function useCreateTask(): UseMutationResult<Task, Error, { title: string; description: string; priority: string }> {
  return useMutation<Task, Error, { title: string; description: string; priority: string }>({
    mutationFn: ({ title, description, priority }) => taskApi.createTask(title, description, priority)
  });
}

export function useUpdateTask(): UseMutationResult<Task, Error, { id: string; updates: Partial<Task> }> {
  return useMutation<Task, Error, { id: string; updates: Partial<Task> }>({
    mutationFn: ({ id, updates }) => taskApi.updateTask(id, updates)
  });
}

export function useDeleteTask(): UseMutationResult<void, Error, string> {
  return useMutation({ mutationFn: taskApi.deleteTask });
}

export function useGenerateSubtasks(): UseMutationResult<string[], Error, { taskId: string; description: string }> {
  return useMutation<string[], Error, { taskId: string; description: string }>({
    mutationFn: ({ taskId, description }) => taskApi.generateSubtasks(taskId, description)
  });
}

export function useProviders(): UseQueryResult<ModelProvider[]> {
  return useQuery({ queryKey: ["providers"], queryFn: modelApi.getProviders });
}

export function useModels(providerId?: string): UseQueryResult<Model[]> {
  return useQuery({ queryKey: ["models", providerId], queryFn: () => modelApi.getModels(providerId) });
}

export function useUpdateProviderConfig(): UseMutationResult<void, Error, { providerId: string; config: { apiKey?: string; baseUrl?: string } }> {
  return useMutation<void, Error, { providerId: string; config: { apiKey?: string; baseUrl?: string } }>({
    mutationFn: ({ providerId, config }) => modelApi.updateProviderConfig(providerId, config)
  });
}

export function useSetDefaultModel(): UseMutationResult<void, Error, string> {
  return useMutation({ mutationFn: modelApi.setDefaultModel });
}

export function useTestModel(): UseMutationResult<{ latency: number; success: boolean }, Error, string> {
  return useMutation({ mutationFn: modelApi.testModel });
}

export function useSettings(): UseQueryResult<AppSettings> {
  return useQuery({ queryKey: ["settings"], queryFn: settingsApi.get });
}

export function useUpdateSettings(): UseMutationResult<AppSettings, Error, Partial<AppSettings>> {
  return useMutation({ mutationFn: settingsApi.update });
}

export function useResetSettings(): UseMutationResult<AppSettings, Error, void> {
  return useMutation({ mutationFn: settingsApi.reset });
}

export function useHealth(): UseQueryResult<{ status: string; version: string }> {
  return useQuery({ queryKey: ["health"], queryFn: systemApi.getHealth });
}
