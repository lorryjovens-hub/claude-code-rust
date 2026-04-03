import { useState } from "react";
import { cn } from "@/lib/utils";
import { useStore } from "@/hooks/useStore";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  MessageSquare,
  CheckSquare,
  Brain,
  Settings,
  History,
  Menu,
  X,
  Plus,
  ChevronLeft,
  ChevronRight,
} from "lucide-react";
import type { ViewType } from "@/types";

const sidebarItems: { id: ViewType; label: string; icon: React.ElementType }[] = [
  { id: "chat", label: "对话", icon: MessageSquare },
  { id: "tasks", label: "任务", icon: CheckSquare },
  { id: "models", label: "模型", icon: Brain },
  { id: "history", label: "历史", icon: History },
  { id: "settings", label: "设置", icon: Settings },
];

interface LayoutProps {
  children: React.ReactNode;
}

export function Layout({ children }: LayoutProps) {
  const { currentView, setCurrentView, sidebarOpen, setSidebarOpen } = useStore();
  const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false);

  return (
    <div className="flex h-screen bg-background">
      {/* Desktop Sidebar */}
      <aside
        className={cn(
          "hidden md:flex flex-col border-r bg-card transition-all duration-300",
          sidebarOpen ? "w-64" : "w-16"
        )}
      >
        {/* Logo */}
        <div className="flex items-center justify-between p-4 border-b">
          {sidebarOpen ? (
            <div className="flex items-center gap-2">
              <div className="w-8 h-8 rounded-lg bg-claude-orange flex items-center justify-center">
                <span className="text-white font-bold text-sm">CC</span>
              </div>
              <span className="font-semibold text-lg">Claude Code</span>
            </div>
          ) : (
            <div className="w-8 h-8 rounded-lg bg-claude-orange flex items-center justify-center mx-auto">
              <span className="text-white font-bold text-sm">CC</span>
            </div>
          )}
          <Button
            variant="ghost"
            size="icon"
            className={cn("h-8 w-8", !sidebarOpen && "hidden")}
            onClick={() => setSidebarOpen(false)}
          >
            <ChevronLeft className="h-4 w-4" />
          </Button>
        </div>

        {/* New Chat Button */}
        <div className="p-3">
          <Button
            className={cn(
              "w-full bg-claude-orange hover:bg-claude-orange-dark",
              !sidebarOpen && "px-2"
            )}
          >
            <Plus className="h-4 w-4" />
            {sidebarOpen && <span className="ml-2">新对话</span>}
          </Button>
        </div>

        {/* Navigation */}
        <ScrollArea className="flex-1 px-3">
          <nav className="space-y-1">
            {sidebarItems.map((item) => {
              const Icon = item.icon;
              const isActive = currentView === item.id;
              return (
                <Button
                  key={item.id}
                  variant={isActive ? "secondary" : "ghost"}
                  className={cn(
                    "w-full justify-start",
                    isActive && "bg-secondary",
                    !sidebarOpen && "justify-center px-2"
                  )}
                  onClick={() => setCurrentView(item.id)}
                >
                  <Icon className="h-4 w-4" />
                  {sidebarOpen && <span className="ml-2">{item.label}</span>}
                </Button>
              );
            })}
          </nav>
        </ScrollArea>

        {/* Toggle Sidebar Button (when collapsed) */}
        {!sidebarOpen && (
          <div className="p-3 border-t">
            <Button
              variant="ghost"
              size="icon"
              className="w-full"
              onClick={() => setSidebarOpen(true)}
            >
              <ChevronRight className="h-4 w-4" />
            </Button>
          </div>
        )}
      </aside>

      {/* Mobile Header */}
      <div className="md:hidden fixed top-0 left-0 right-0 z-50 bg-card border-b">
        <div className="flex items-center justify-between p-4">
          <div className="flex items-center gap-2">
            <div className="w-8 h-8 rounded-lg bg-claude-orange flex items-center justify-center">
              <span className="text-white font-bold text-sm">CC</span>
            </div>
            <span className="font-semibold">Claude Code</span>
          </div>
          <Button
            variant="ghost"
            size="icon"
            onClick={() => setIsMobileMenuOpen(!isMobileMenuOpen)}
          >
            {isMobileMenuOpen ? (
              <X className="h-5 w-5" />
            ) : (
              <Menu className="h-5 w-5" />
            )}
          </Button>
        </div>

        {/* Mobile Menu */}
        {isMobileMenuOpen && (
          <div className="border-t">
            <nav className="p-2 space-y-1">
              {sidebarItems.map((item) => {
                const Icon = item.icon;
                const isActive = currentView === item.id;
                return (
                  <Button
                    key={item.id}
                    variant={isActive ? "secondary" : "ghost"}
                    className={cn(
                      "w-full justify-start",
                      isActive && "bg-secondary"
                    )}
                    onClick={() => {
                      setCurrentView(item.id);
                      setIsMobileMenuOpen(false);
                    }}
                  >
                    <Icon className="h-4 w-4 mr-2" />
                    {item.label}
                  </Button>
                );
              })}
            </nav>
          </div>
        )}
      </div>

      {/* Main Content */}
      <main className="flex-1 overflow-hidden md:pt-0 pt-16">
        {children}
      </main>
    </div>
  );
}
