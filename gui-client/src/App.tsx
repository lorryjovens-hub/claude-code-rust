import { useState } from "react";
import { Outlet, useLocation } from "react-router-dom";
import { Sidebar, SidebarContent, SidebarHeader, SidebarFooter } from "./components/ui/sidebar";
import { Button } from "./components/ui/button";
import { Input } from "./components/ui/input";
import { Search, MessageSquare, CheckSquare, Settings, Brain, ChevronLeft, ChevronRight } from "lucide-react";

export function App() {
  const [sidebarOpen, setSidebarOpen] = useState(true);
  const location = useLocation();

  const navItems = [
    { id: "", label: "对话", icon: MessageSquare, href: "/" },
    { id: "tasks", label: "任务", icon: CheckSquare, href: "/tasks" },
    { id: "models", label: "模型", icon: Brain, href: "/models" },
    { id: "settings", label: "设置", icon: Settings, href: "/settings" },
  ];

  const isActive = (path: string) => {
    return location.pathname === path;
  };

  return (
    <div className="flex h-screen bg-background">
      <Sidebar open={sidebarOpen} onOpenChange={setSidebarOpen}>
        <SidebarHeader className="border-b p-4">
          <div className="flex items-center gap-2">
            <div className="w-8 h-8 rounded-lg bg-claude-orange flex items-center justify-center">
              <span className="text-white font-bold text-sm">CC</span>
            </div>
            {sidebarOpen && (
              <span className="font-semibold text-lg">Claude Code</span>
            )}
          </div>
        </SidebarHeader>
        <SidebarContent className="p-4">
          {sidebarOpen && (
            <div className="mb-4">
              <div className="relative">
                <Search className="absolute left-3 top-2.5 h-4 w-4 text-muted-foreground" />
                <Input
                  type="search"
                  placeholder="搜索..."
                  className="w-full pl-10"
                />
              </div>
            </div>
          )}
          <nav className="space-y-1">
            {navItems.map((item) => (
              <Button
                key={item.id}
                variant="ghost"
                className={`w-full justify-start ${isActive(item.href) ? 'bg-secondary' : ''}`}
                onClick={() => {
                  window.location.href = item.href;
                }}
              >
                <item.icon className="mr-2 h-5 w-5" />
                {sidebarOpen && item.label}
              </Button>
            ))}
          </nav>
        </SidebarContent>
        <SidebarFooter className="border-t p-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <div className="w-8 h-8 rounded-full bg-secondary flex items-center justify-center">
                <span className="text-sm font-medium">U</span>
              </div>
              {sidebarOpen && (
                <div>
                  <p className="text-sm font-medium">用户</p>
                  <p className="text-xs text-muted-foreground">user@example.com</p>
                </div>
              )}
            </div>
            <Button
              variant="ghost"
              size="icon"
              onClick={() => setSidebarOpen(!sidebarOpen)}
            >
              {sidebarOpen ? <ChevronLeft className="h-5 w-5" /> : <ChevronRight className="h-5 w-5" />}
            </Button>
          </div>
        </SidebarFooter>
      </Sidebar>
      <div className="flex-1 flex flex-col overflow-hidden">
        <div className="flex-1 overflow-auto">
          <Outlet />
        </div>
      </div>
    </div>
  );
}
