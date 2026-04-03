import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/utils";

const sidebarVariants = cva(
  "fixed inset-y-0 left-0 z-50 flex h-full w-64 flex-col border-r bg-background",
  {
    variants: {
      open: {
        true: "translate-x-0",
        false: "-translate-x-full",
      },
    },
    defaultVariants: {
      open: true,
    },
  }
);

export interface SidebarProps
  extends React.HTMLAttributes<HTMLDivElement>,
    VariantProps<typeof sidebarVariants> {
  onOpenChange?: (open: boolean) => void;
}

export function Sidebar({ className, open = true, onOpenChange, children, ...props }: SidebarProps) {
  return (
    <div
      className={cn(sidebarVariants({ open }), className)}
      onMouseEnter={() => onOpenChange?.(true)}
      onMouseLeave={() => onOpenChange?.(false)}
      {...props}
    >
      {children}
    </div>
  );
}

export function SidebarHeader({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      className={cn("flex h-16 items-center border-b px-4", className)}
      {...props}
    />
  );
}

export function SidebarContent({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return (
    <div className={cn("flex-1 overflow-y-auto", className)} {...props} />
  );
}

export function SidebarFooter({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      className={cn("flex h-16 items-center border-t px-4", className)}
      {...props}
    />
  );
}
