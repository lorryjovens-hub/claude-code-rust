import { useState } from "react";
import { useProviders, useModels, useUpdateProviderConfig, useSetDefaultModel, useTestModel } from "@/hooks/useApi";
import { useStore } from "@/hooks/useStore";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Loader2, Settings, TestTube, Save, Edit2 } from "lucide-react";
import { toast } from "sonner";
import type { Model, ModelProvider } from "@/lib/api";

export function ModelPage() {
  const { settings, setSettings } = useStore();
  const [editingProvider, setEditingProvider] = useState<string | null>(null);
  const [apiKey, setApiKey] = useState("");
  const [baseUrl, setBaseUrl] = useState("");

  const { data: providers = [], refetch: refetchProviders } = useProviders();
  const { data: allModels = [] } = useModels();
  const updateProviderConfig = useUpdateProviderConfig();
  const setDefaultModel = useSetDefaultModel();
  const testModel = useTestModel();

  const handleEnableProvider = (provider: ModelProvider) => {
    // In a real implementation, this would toggle the provider's enabled status
    toast.info(`${provider.name} ${provider.is_enabled ? '已禁用' : '已启用'}`);
  };

  const handleEditProvider = (provider: ModelProvider) => {
    setEditingProvider(provider.id);
    setApiKey(provider.api_key || "");
    setBaseUrl(provider.base_url || "");
  };

  const handleSaveProvider = async (provider: ModelProvider) => {
    try {
      await updateProviderConfig.mutateAsync({
        providerId: provider.id,
        config: { apiKey, baseUrl },
      });
      toast.success(`${provider.name} 配置已保存`);
      setEditingProvider(null);
      refetchProviders();
    } catch (error) {
      // Error already handled by mutation
    }
  };

  const handleTestModel = async (model: Model) => {
    try {
      const result = await testModel.mutateAsync(model.id);
      toast.success(`${model.name} 测试成功，延迟: ${result.latency}ms`);
    } catch (error: any) {
      toast.error(`${model.name} 测试失败: ${error.message}`);
    }
  };

  const handleSetDefaultModel = async (model: Model) => {
    try {
      await setDefaultModel.mutateAsync(model.id);
      setSettings({ ...settings, default_model: model.id });
      toast.success(`${model.name} 已设置为默认模型`);
    } catch (error) {
      // Error already handled by mutation
    }
  };

  return (
    <div className="flex h-full flex-col p-6">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-2xl font-bold">模型管理</h1>
          <p className="text-muted-foreground">管理AI模型提供商和模型配置</p>
        </div>
      </div>

      <div className="flex-1 space-y-6">
        {/* Provider Tabs */}
        <Tabs defaultValue="all" className="w-full">
          <TabsList className="grid w-full grid-cols-6">
            <TabsTrigger value="all">全部</TabsTrigger>
            {providers.map((provider: ModelProvider) => (
              <TabsTrigger key={provider.id} value={provider.id}>
                {provider.name}
              </TabsTrigger>
            ))}
          </TabsList>

          <TabsContent value="all" className="mt-4">
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              {allModels.map((model: Model) => (
                <Card key={model.id} className="overflow-hidden">
                  <CardHeader className="pb-2">
                    <div className="flex items-center justify-between">
                      <div>
                        <CardTitle className="text-lg">{model.name}</CardTitle>
                        <CardDescription>{model.provider}</CardDescription>
                      </div>
                      <div className="flex items-center gap-2">
                        <Button
                          variant="ghost"
                          size="icon"
                          onClick={() => handleTestModel(model)}
                          disabled={testModel.isPending}
                        >
                          {testModel.isPending ? (
                            <Loader2 className="w-4 h-4 animate-spin" />
                          ) : (
                            <TestTube className="w-4 h-4" />
                          )}
                        </Button>
                        <Button
                          size="sm"
                          className={model.is_default ? "bg-claude-orange hover:bg-claude-orange-dark" : ""}
                          onClick={() => handleSetDefaultModel(model)}
                          disabled={setDefaultModel.isPending}
                        >
                          {model.is_default ? "默认" : "设为默认"}
                        </Button>
                      </div>
                    </div>
                  </CardHeader>
                  <CardContent className="pb-2">
                    <p className="text-sm text-muted-foreground mb-4">{model.description}</p>
                    <div className="space-y-2 text-sm">
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
                  </CardFooter>
                </Card>
              ))}
            </div>
          </TabsContent>

          {providers.map((provider: ModelProvider) => (
            <TabsContent key={provider.id} value={provider.id} className="mt-4">
              <div className="space-y-4">
                {/* Provider Configuration */}
                <Card>
                  <CardHeader>
                    <CardTitle>{provider.name} 配置</CardTitle>
                    <CardDescription>管理API密钥和基础URL</CardDescription>
                  </CardHeader>
                  <CardContent className="space-y-4">
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-2">
                        <Switch
                          id={`enable-${provider.id}`}
                          checked={provider.is_enabled}
                          onCheckedChange={() => handleEnableProvider(provider)}
                        />
                        <Label htmlFor={`enable-${provider.id}`}>启用 {provider.name}</Label>
                      </div>
                      {provider.is_enabled && (
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => handleEditProvider(provider)}
                          className="gap-1"
                        >
                          <Settings className="w-4 h-4" />
                          配置
                        </Button>
                      )}
                    </div>

                    {editingProvider === provider.id && (
                      <div className="space-y-4 p-4 bg-muted rounded-lg">
                        <div className="space-y-2">
                          <Label htmlFor={`api-key-${provider.id}`}>API 密钥</Label>
                          <Input
                            id={`api-key-${provider.id}`}
                            type="password"
                            value={apiKey}
                            onChange={(e) => setApiKey(e.target.value)}
                            placeholder="输入API密钥"
                          />
                        </div>
                        <div className="space-y-2">
                          <Label htmlFor={`base-url-${provider.id}`}>基础 URL</Label>
                          <Input
                            id={`base-url-${provider.id}`}
                            value={baseUrl}
                            onChange={(e) => setBaseUrl(e.target.value)}
                            placeholder="输入基础URL (可选)"
                          />
                        </div>
                        <div className="flex gap-2">
                          <Button
                            onClick={() => handleSaveProvider(provider)}
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
                        </div>
                      </div>
                    )}
                  </CardContent>
                </Card>

                {/* Provider Models */}
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  {provider.models.map((model: Model) => (
                    <Card key={model.id} className="overflow-hidden">
                      <CardHeader className="pb-2">
                        <div className="flex items-center justify-between">
                          <div>
                            <CardTitle className="text-lg">{model.name}</CardTitle>
                            <CardDescription>{model.description}</CardDescription>
                          </div>
                          <Button
                            size="sm"
                            className={model.is_default ? "bg-claude-orange hover:bg-claude-orange-dark" : ""}
                            onClick={() => handleSetDefaultModel(model)}
                            disabled={setDefaultModel.isPending}
                          >
                            {model.is_default ? "默认" : "设为默认"}
                          </Button>
                        </div>
                      </CardHeader>
                      <CardContent className="pb-2">
                        <div className="space-y-2 text-sm">
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
                        </div>
                      </CardContent>
                      <CardFooter className="flex justify-between items-center border-t pt-4">
                        <span className={`text-xs ${model.is_available ? 'text-green-500' : 'text-red-500'}`}>
                          {model.is_available ? '可用' : '不可用'}
                        </span>
                        <Button
                          variant="ghost"
                          size="icon"
                          onClick={() => handleTestModel(model)}
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
              </div>
            </TabsContent>
          ))}
        </Tabs>

        {/* Provider Status */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-4 mt-6">
          {providers.map((provider: ModelProvider) => (
            <Card key={provider.id} className="border-l-4 border-l-emerald-500">
              <CardContent className="p-4">
                <div className="flex items-center justify-between mb-2">
                  <h3 className="font-medium">{provider.name}</h3>
                  <span className={`text-xs px-2 py-1 rounded ${provider.is_enabled ? 'bg-green-100 text-green-800' : 'bg-gray-100 text-gray-800'}`}>
                    {provider.is_enabled ? '已启用' : '已禁用'}
                  </span>
                </div>
                <p className="text-sm text-muted-foreground">{provider.models.length} 个模型</p>
                <div className="mt-2 flex items-center gap-2">
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => handleEditProvider(provider)}
                    className="gap-1 text-xs"
                  >
                    <Edit2 className="w-3 h-3" />
                    编辑
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => handleTestModel(provider.models[0])}
                    disabled={testModel.isPending || provider.models.length === 0}
                    className="gap-1 text-xs"
                  >
                    <TestTube className="w-3 h-3" />
                    测试
                  </Button>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>

        {/* System Status */}
        <Card className="border-l-4 border-l-blue-500">
          <CardHeader>
            <CardTitle>系统状态</CardTitle>
            <CardDescription>当前默认模型和系统配置</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <span>默认模型:</span>
                <span className="font-medium">
                  {allModels.find((m: Model) => m.id === settings.default_model)?.name || '未设置'}
                </span>
              </div>
              <div className="flex items-center justify-between">
                <span>流式响应:</span>
                <span>{settings.stream_response ? '启用' : '禁用'}</span>
              </div>
              <div className="flex items-center justify-between">
                <span>思考模式:</span>
                <span>{settings.show_thinking ? '启用' : '禁用'}</span>
              </div>
              <div className="flex items-center justify-between">
                <span>最大上下文消息:</span>
                <span>{settings.max_context_messages}</span>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
