import { useState } from "react";
import { useProviders, useModels, useSetDefaultModel, useTestModel, useUpdateProviderConfig } from "@/hooks/useApi";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Loader2, TestTube, Save, Edit2, Brain } from "lucide-react";
import { toast } from "sonner";

export function ModelsPage() {
  const [editingProvider, setEditingProvider] = useState<string | null>(null);
  const [apiKey, setApiKey] = useState("");
  const [baseUrl, setBaseUrl] = useState("");
  const [selectedProvider, setSelectedProvider] = useState<string>("");

  const { data: providers = [] } = useProviders();
  const { data: models = [] } = useModels(selectedProvider);
  const setDefaultModel = useSetDefaultModel();
  const testModel = useTestModel();
  const updateProviderConfig = useUpdateProviderConfig();

  const handleSetDefaultModel = async (modelId: string) => {
    try {
      await setDefaultModel.mutateAsync(modelId);
      toast.success("默认模型已更新");
    } catch (error: any) {
      toast.error(`设置默认模型失败: ${error.message}`);
    }
  };

  const handleTestModel = async (modelId: string) => {
    try {
      const result = await testModel.mutateAsync(modelId);
      toast.success(`模型测试成功，延迟: ${result.latency}ms`);
    } catch (error: any) {
      toast.error(`模型测试失败: ${error.message}`);
    }
  };

  const handleUpdateProvider = async (providerId: string) => {
    try {
      await updateProviderConfig.mutateAsync({ providerId, config: { apiKey, baseUrl } });
      toast.success("提供商配置已更新");
      setEditingProvider(null);
    } catch (error: any) {
      toast.error(`更新提供商配置失败: ${error.message}`);
    }
  };

  return (
    <div className="flex h-full flex-col p-6">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-2xl font-bold flex items-center gap-2">
            <Brain className="h-6 w-6" />
            模型管理
          </h1>
          <p className="text-muted-foreground">管理AI模型和提供商配置</p>
        </div>
      </div>

      <div className="flex-1 space-y-6">
        {/* Provider Selector */}
        <div className="flex flex-wrap gap-4">
          {providers.map((provider) => (
            <Button
              key={provider.id}
              variant={selectedProvider === provider.id ? "default" : "ghost"}
              onClick={() => setSelectedProvider(provider.id)}
              className="flex items-center gap-2"
            >
              {provider.name}
            </Button>
          ))}
        </div>

        {/* Models List */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {models.map((model) => (
            <Card key={model.id} className="overflow-hidden">
              <CardHeader className="pb-2">
                <div className="flex items-center justify-between">
                  <div>
                    <CardTitle>{model.name}</CardTitle>
                    <CardDescription>{model.description}</CardDescription>
                  </div>
                  <Button
                    size="sm"
                    className={model.is_default ? "bg-claude-orange hover:bg-claude-orange-dark" : ""}
                    onClick={() => handleSetDefaultModel(model.id)}
                  >
                    {model.is_default ? "默认" : "设为默认"}
                  </Button>
                </div>
              </CardHeader>
              <CardContent className="pb-2">
                <div className="space-y-2 text-sm">
                  <div className="flex justify-between">
                    <span>提供商:</span>
                    <span>{model.provider}</span>
                  </div>
                  <div className="flex justify-between">
                    <span>上下文窗口:</span>
                    <span>{model.context_window.toLocaleString()} tokens</span>
                  </div>
                  <div className="flex justify-between">
                    <span>最大输出:</span>
                    <span>{model.max_tokens.toLocaleString()} tokens</span>
                  </div>
                  <div className="flex justify-between">
                    <span>价格:</span>
                    <span>
                      ${model.pricing.input.toFixed(4)}/1K输入, ${model.pricing.output.toFixed(4)}/1K输出
                    </span>
                  </div>
                  <div className="flex flex-wrap gap-1 mt-2">
                    {model.capabilities.map((capability) => (
                      <span key={capability} className="px-2 py-1 bg-secondary rounded text-xs">
                        {capability}
                      </span>
                    ))}
                  </div>
                </div>
              </CardContent>
              <CardFooter className="flex justify-between items-center border-t pt-4">
                <span className={`text-xs ${model.is_available ? 'text-green-500' : 'text-red-500'}`}>
                  {model.is_available ? '可用' : '不可用'}
                </span>
                <Button
                  variant="ghost"
                  size="icon"
                  onClick={() => handleTestModel(model.id)}
                  disabled={testModel.isPending}
                >
                  {testModel.isPending ? (
                    <Loader2 className="w-4 h-4 animate-spin" />
                  ) : (
                    <TestTube className="w-4 h-4" />
                  )}
                </Button>
              </CardFooter>
            </Card>
          ))}
        </div>

        {/* Provider Configuration */}
        {editingProvider && (
          <Card>
            <CardHeader>
              <CardTitle>提供商配置</CardTitle>
              <CardDescription>管理API密钥和基础URL</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="api-key">API 密钥</Label>
                <Input
                  id="api-key"
                  type="password"
                  value={apiKey}
                  onChange={(e) => setApiKey(e.target.value)}
                  placeholder="输入API密钥"
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="base-url">基础 URL</Label>
                <Input
                  id="base-url"
                  value={baseUrl}
                  onChange={(e) => setBaseUrl(e.target.value)}
                  placeholder="输入基础URL (可选)"
                />
              </div>
            </CardContent>
            <CardFooter className="flex gap-2">
              <Button
                onClick={() => handleUpdateProvider(editingProvider)}
                disabled={updateProviderConfig.isPending}
                className="flex-1"
              >
                {updateProviderConfig.isPending ? (
                  <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                ) : (
                  <Save className="w-4 h-4 mr-2" />
                )}
                保存
              </Button>
              <Button
                variant="ghost"
                onClick={() => setEditingProvider(null)}
              >
                取消
              </Button>
            </CardFooter>
          </Card>
        )}

        {/* Provider List */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {providers.map((provider) => (
            <Card key={provider.id}>
              <CardHeader className="pb-2">
                <div className="flex items-center justify-between">
                  <CardTitle>{provider.name}</CardTitle>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => setEditingProvider(provider.id)}
                    className="gap-1"
                  >
                    <Edit2 className="w-4 h-4" />
                    编辑
                  </Button>
                </div>
              </CardHeader>
              <CardContent>
                <div className="space-y-2 text-sm">
                  <div className="flex items-center gap-2">
                    <Switch
                      id={`enable-${provider.id}`}
                      checked={provider.is_enabled}
                      onCheckedChange={() => {}}
                    />
                    <Label htmlFor={`enable-${provider.id}`}>启用</Label>
                  </div>
                  <div>
                    <Label>模型数量:</Label>
                    <p>{provider.models.length}</p>
                  </div>
                  <div>
                    <Label>API 密钥:</Label>
                    <p className="text-muted-foreground">
                      {provider.api_key ? '已配置' : '未配置'}
                    </p>
                  </div>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      </div>
    </div>
  );
}
