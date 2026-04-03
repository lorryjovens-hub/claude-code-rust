import { useState } from "react";
import { useTasks, useCreateTask, useUpdateTask, useDeleteTask, useGenerateSubtasks } from "@/hooks/useApi";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Plus,
  CheckCircle2,
  Circle,
  Clock,
  AlertCircle,
  Trash2,
  Sparkles,
  Loader2,
} from "lucide-react";
import type { Task, SubTask } from "@/lib/api";
import { generateId } from "@/lib/utils";

export function TaskPage() {
  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false);

  const { data: tasks = [], isLoading } = useTasks();
  const createTask = useCreateTask();
  const updateTask = useUpdateTask();
  const deleteTask = useDeleteTask();
  const generateSubtasks = useGenerateSubtasks();

  const handleCreateTask = async (taskData: { title: string; description: string; priority: string }) => {
    await createTask.mutateAsync({
      title: taskData.title,
      description: taskData.description,
      priority: taskData.priority,
    });
    setIsCreateDialogOpen(false);
  };

  const handleUpdateTask = async (taskId: string, updates: Partial<Task>) => {
    await updateTask.mutateAsync({ id: taskId, updates });
  };

  const handleDeleteTask = async (taskId: string) => {
    await deleteTask.mutateAsync(taskId);
  };

  const handleGenerateSubtasks = async (task: Task) => {
    const subtaskTitles = await generateSubtasks.mutateAsync({
      taskId: task.id,
      description: task.description,
    });
    
    const subtasks: SubTask[] = subtaskTitles.map((title) => ({
      id: generateId(),
      title,
      completed: false,
    }));
    
    await updateTask.mutateAsync({ id: task.id, updates: { subtasks } });
  };

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b">
        <h1 className="text-2xl font-semibold">任务管理</h1>
        <Dialog open={isCreateDialogOpen} onOpenChange={setIsCreateDialogOpen}>
          <DialogTrigger asChild>
            <Button className="bg-claude-orange hover:bg-claude-orange-dark">
              <Plus className="w-4 h-4 mr-2" />
              新建任务
            </Button>
          </DialogTrigger>
          <DialogContent className="max-w-lg">
            <DialogHeader>
              <DialogTitle>创建新任务</DialogTitle>
            </DialogHeader>
            <TaskForm
              onSubmit={handleCreateTask}
              isPending={createTask.isPending}
            />
          </DialogContent>
        </Dialog>
      </div>

      {/* Task List */}
      <ScrollArea className="flex-1 p-4">
        <div className="grid gap-4 max-w-4xl mx-auto">
          {isLoading ? (
            <div className="flex items-center justify-center py-12">
              <Loader2 className="w-8 h-8 animate-spin text-claude-orange" />
            </div>
          ) : tasks.length === 0 ? (
            <div className="text-center py-12">
              <div className="w-16 h-16 rounded-full bg-muted flex items-center justify-center mx-auto mb-4">
                <CheckCircle2 className="w-8 h-8 text-muted-foreground" />
              </div>
              <h3 className="text-lg font-medium mb-2">暂无任务</h3>
              <p className="text-muted-foreground">
                点击上方按钮创建你的第一个任务
              </p>
            </div>
          ) : (
            tasks.map((task) => (
              <TaskCard
                key={task.id}
                task={task}
                onUpdate={handleUpdateTask}
                onDelete={handleDeleteTask}
                onGenerateSubtasks={handleGenerateSubtasks}
                isUpdating={updateTask.isPending}
                isDeleting={deleteTask.isPending}
                isGenerating={generateSubtasks.isPending}
              />
            ))
          )}
        </div>
      </ScrollArea>
    </div>
  );
}

interface TaskCardProps {
  task: Task;
  onUpdate: (taskId: string, updates: Partial<Task>) => Promise<void>;
  onDelete: (taskId: string) => Promise<void>;
  onGenerateSubtasks: (task: Task) => Promise<void>;
  isUpdating: boolean;
  isDeleting: boolean;
  isGenerating: boolean;
}

function TaskCard({
  task,
  onUpdate,
  onDelete,
  onGenerateSubtasks,
  isUpdating,
  isDeleting,
  isGenerating,
}: TaskCardProps) {
  const getStatusIcon = (status: string) => {
    switch (status) {
      case "completed":
        return <CheckCircle2 className="w-5 h-5 text-green-500" />;
      case "in_progress":
        return <Clock className="w-5 h-5 text-blue-500" />;
      case "failed":
        return <AlertCircle className="w-5 h-5 text-red-500" />;
      default:
        return <Circle className="w-5 h-5 text-gray-400" />;
    }
  };

  const getPriorityColor = (priority: string) => {
    switch (priority) {
      case "high":
        return "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200";
      case "medium":
        return "bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200";
      case "low":
        return "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200";
      default:
        return "";
    }
  };

  const handleToggleStatus = async () => {
    const newStatus = task.status === "completed" ? "pending" : "completed";
    await onUpdate(task.id, { status: newStatus });
  };

  const handleToggleSubtask = async (subtaskId: string) => {
    const updatedSubtasks = task.subtasks.map((st) =>
      st.id === subtaskId ? { ...st, completed: !st.completed } : st
    );
    const completedCount = updatedSubtasks.filter((st) => st.completed).length;
    const progress = Math.round((completedCount / updatedSubtasks.length) * 100);
    
    await onUpdate(task.id, {
      subtasks: updatedSubtasks,
      progress,
      status: progress === 100 ? "completed" : progress > 0 ? "in_progress" : "pending",
    });
  };

  return (
    <Card className="hover:shadow-md transition-shadow">
      <CardHeader className="pb-3">
        <div className="flex items-start justify-between">
          <div className="flex items-start gap-3">
            <button
              onClick={handleToggleStatus}
              disabled={isUpdating}
              className="mt-1 disabled:opacity-50"
            >
              {isUpdating ? (
                <Loader2 className="w-5 h-5 animate-spin" />
              ) : (
                getStatusIcon(task.status)
              )}
            </button>
            <div>
              <CardTitle className="text-lg font-medium">
                {task.title}
              </CardTitle>
              <div className="flex items-center gap-2 mt-1">
                <Badge className={getPriorityColor(task.priority)}>
                  {task.priority === "high"
                    ? "高优先级"
                    : task.priority === "medium"
                    ? "中优先级"
                    : "低优先级"}
                </Badge>
                <span className="text-sm text-muted-foreground">
                  {task.subtasks.filter((st) => st.completed).length} /{" "}
                  {task.subtasks.length} 子任务
                </span>
              </div>
            </div>
          </div>
          <div className="flex items-center gap-1">
            <Button
              variant="ghost"
              size="icon"
              onClick={() => onGenerateSubtasks(task)}
              disabled={isGenerating}
            >
              {isGenerating ? (
                <Loader2 className="w-4 h-4 animate-spin" />
              ) : (
                <Sparkles className="w-4 h-4" />
              )}
            </Button>
            <Button
              variant="ghost"
              size="icon"
              onClick={() => onDelete(task.id)}
              disabled={isDeleting}
            >
              {isDeleting ? (
                <Loader2 className="w-4 h-4 animate-spin text-destructive" />
              ) : (
                <Trash2 className="w-4 h-4 text-destructive" />
              )}
            </Button>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        <p className="text-muted-foreground mb-4">{task.description}</p>

        {/* Progress Bar */}
        <div className="w-full bg-muted rounded-full h-2 mb-4">
          <div
            className="bg-claude-orange h-2 rounded-full transition-all"
            style={{ width: `${task.progress}%` }}
          />
        </div>

        {/* Subtasks */}
        {task.subtasks.length > 0 && (
          <div className="space-y-2 mt-4">
            <h4 className="text-sm font-medium">子任务</h4>
            {task.subtasks.map((subtask) => (
              <div
                key={subtask.id}
                className="flex items-center gap-2 text-sm"
              >
                <button
                  onClick={() => handleToggleSubtask(subtask.id)}
                  disabled={isUpdating}
                  className="disabled:opacity-50"
                >
                  {subtask.completed ? (
                    <CheckCircle2 className="w-4 h-4 text-green-500" />
                  ) : (
                    <Circle className="w-4 h-4 text-gray-400" />
                  )}
                </button>
                <span
                  className={cn(
                    subtask.completed && "line-through text-muted-foreground"
                  )}
                >
                  {subtask.title}
                </span>
              </div>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}

interface TaskFormProps {
  onSubmit: (data: { title: string; description: string; priority: string }) => Promise<void>;
  isPending: boolean;
}

function TaskForm({ onSubmit, isPending }: TaskFormProps) {
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [priority, setPriority] = useState<Task["priority"]>("medium");

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    await onSubmit({ title, description, priority });
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <div>
        <label className="text-sm font-medium mb-2 block">任务标题</label>
        <Input
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          placeholder="输入任务标题"
          required
          disabled={isPending}
        />
      </div>
      <div>
        <label className="text-sm font-medium mb-2 block">任务描述</label>
        <Textarea
          value={description}
          onChange={(e) => setDescription(e.target.value)}
          placeholder="输入任务描述"
          rows={3}
          disabled={isPending}
        />
      </div>
      <div>
        <label className="text-sm font-medium mb-2 block">优先级</label>
        <Select
          value={priority}
          onValueChange={(v) => setPriority(v as Task["priority"])}
          disabled={isPending}
        >
          <SelectTrigger>
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="low">低</SelectItem>
            <SelectItem value="medium">中</SelectItem>
            <SelectItem value="high">高</SelectItem>
          </SelectContent>
        </Select>
      </div>
      <div className="flex justify-end gap-2">
        <Button
          type="submit"
          className="bg-claude-orange hover:bg-claude-orange-dark"
          disabled={isPending}
        >
          {isPending ? (
            <Loader2 className="w-4 h-4 mr-2 animate-spin" />
          ) : null}
          创建任务
        </Button>
      </div>
    </form>
  );
}
