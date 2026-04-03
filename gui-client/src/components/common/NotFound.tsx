import { Button } from "@/components/ui/button";
import { Home } from "lucide-react";

export function NotFound() {
  return (
    <div className="flex min-h-screen flex-col items-center justify-center gap-8 px-4">
      <div className="text-center space-y-4">
        <h1 className="text-6xl font-bold">404</h1>
        <h2 className="text-2xl font-semibold">页面未找到</h2>
        <p className="text-muted-foreground max-w-md mx-auto">
          抱歉，您访问的页面不存在或已被删除。
        </p>
        <Button
          className="mt-4 bg-claude-orange hover:bg-claude-orange-dark"
          onClick={() => window.location.href = "/"}
        >
          <Home className="w-4 h-4 mr-2" />
          返回首页
        </Button>
      </div>
    </div>
  );
}
