import { useSettings, useUpdateSettings, useResetSettings } from "@/hooks/useApi";
import { useStore } from "@/hooks/useStore";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Switch } from "@/components/ui/switch";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Settings,
  Moon,
  Sun,
  Monitor,
  Keyboard,
  Globe,
  RotateCcw,
  Info,
  Loader2,
} from "lucide-react";
import type { AppSettings } from "@/lib/api";

export function SettingsPage() {
  const { theme, setTheme } = useStore();
  const { data: settings, isLoading } = useSettings();
  const updateSettings = useUpdateSettings();
  const resetSettings = useResetSettings();

  const handleUpdateSettings = async (newSettings: Partial<AppSettings>) => {
    if (!settings) return;
    await updateSettings.mutateAsync(newSettings);
  };

  const handleResetSettings = async () => {
    await resetSettings.mutateAsync();
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
        <Loader2 className="w-8 h-8 animate-spin text-claude-orange" />
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b">
        <h1 className="text-2xl font-semibold">设置</h1>
        <div className="flex gap-2">
          <Button
            variant="outline"
            onClick={handleResetSettings}
            disabled={resetSettings.isPending}
          >
            {resetSettings.isPending ? (
              <Loader2 className="w-4 h-4 mr-2 animate-spin" />
            ) : (
              <RotateCcw className="w-4 h-4 mr-2" />
            )}
            重置
          </Button>
        </div>
      </div>

      {/* Settings Content */}
      <ScrollArea className="flex-1 p-4">
        <div className="max-w-3xl mx-auto space-y-6">
          {/* Appearance */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Monitor className="w-5 h-5" />
                外观
              </CardTitle>
              <CardDescription>自定义应用的外观和主题</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <label className="text-sm font-medium">主题</label>
                  <p className="text-sm text-muted-foreground">选择应用的主题模式</p>
                </div>
                <Select
                  value={theme}
                  onValueChange={(value) => {
                    setTheme(value as typeof theme);
                    handleUpdateSettings({ theme: value });
                  }}
                >
                  <SelectTrigger className="w-32">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="light">
                      <div className="flex items-center gap-2">
                        <Sun className="w-4 h-4" />
                        浅色
                      </div>
                    </SelectItem>
                    <SelectItem value="dark">
                      <div className="flex items-center gap-2">
                        <Moon className="w-4 h-4" />
                        深色
                      </div>
                    </SelectItem>
                    <SelectItem value="system">
                      <div className="flex items-center gap-2">
                        <Monitor className="w-4 h-4" />
                        系统
                      </div>
                    </SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </CardContent>
          </Card>

          {/* Language */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Globe className="w-5 h-5" />
                语言
              </CardTitle>
              <CardDescription>选择应用显示语言</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="flex items-center justify-between">
                <div>
                  <label className="text-sm font-medium">界面语言</label>
                  <p className="text-sm text-muted-foreground">更改应用的显示语言</p>
                </div>
                <Select
                  value={settings?.language || "zh-CN"}
                  onValueChange={(value) => handleUpdateSettings({ language: value })}
                >
                  <SelectTrigger className="w-40">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="zh-CN">简体中文</SelectItem>
                    <SelectItem value="en-US">English</SelectItem>
                    <SelectItem value="ja-JP">日本語</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </CardContent>
          </Card>

          {/* Chat Settings */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Settings className="w-5 h-5" />
                对话设置
              </CardTitle>
              <CardDescription>配置对话行为</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex items-center justify-between">
                <div>
                  <label className="text-sm font-medium">流式响应</label>
                  <p className="text-sm text-muted-foreground">实时显示 AI 回复内容</p>
                </div>
                <Switch
                  checked={settings?.stream_response ?? true}
                  onCheckedChange={(checked) =>
                    handleUpdateSettings({ stream_response: checked })
                  }
                />
              </div>

              <div className="flex items-center justify-between">
                <div>
                  <label className="text-sm font-medium">显示思考过程</label>
                  <p className="text-sm text-muted-foreground">显示 AI 的思考过程</p>
                </div>
                <Switch
                  checked={settings?.show_thinking ?? false}
                  onCheckedChange={(checked) =>
                    handleUpdateSettings({ show_thinking: checked })
                  }
                />
              </div>

              <div className="flex items-center justify-between">
                <div>
                  <label className="text-sm font-medium">自动保存</label>
                  <p className="text-sm text-muted-foreground">自动保存对话历史</p>
                </div>
                <Switch
                  checked={settings?.auto_save ?? true}
                  onCheckedChange={(checked) =>
                    handleUpdateSettings({ auto_save: checked })
                  }
                />
              </div>

              <div className="flex items-center justify-between">
                <div>
                  <label className="text-sm font-medium">最大上下文消息数</label>
                  <p className="text-sm text-muted-foreground">保留在上下文中的最大消息数量</p>
                </div>
                <Input
                  type="number"
                  value={settings?.max_context_messages ?? 20}
                  onChange={(e) =>
                    handleUpdateSettings({
                      max_context_messages: parseInt(e.target.value) || 20,
                    })
                  }
                  className="w-24"
                  min={5}
                  max={50}
                />
              </div>
            </CardContent>
          </Card>

          {/* Shortcuts */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Keyboard className="w-5 h-5" />
                快捷键
              </CardTitle>
              <CardDescription>自定义键盘快捷键</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              {settings?.shortcuts &&
                Object.entries(settings.shortcuts).map(([action, shortcut]) => (
                  <div key={action} className="flex items-center justify-between">
                    <div>
                      <label className="text-sm font-medium">
                        {action === "new-chat" && "新建对话"}
                        {action === "send-message" && "发送消息"}
                        {action === "new-line" && "换行"}
                        {action === "search" && "搜索"}
                      </label>
                    </div>
                    <Badge variant="secondary">{shortcut}</Badge>
                  </div>
                ))}
            </CardContent>
          </Card>

          {/* About */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Info className="w-5 h-5" />
                关于
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-2">
                <div className="flex justify-between">
                  <span className="text-muted-foreground">版本</span>
                  <span>1.0.0</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">构建</span>
                  <span>2024.01.15</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-muted-foreground">Rust Claude Code</span>
                  <span>v0.1.0</span>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
      </ScrollArea>
    </div>
  );
}
