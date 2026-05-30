import React, { useState, useCallback, useEffect, useRef } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import {
  PanelLeftClose,
  PanelLeftOpen,
  PanelRightClose,
  PanelRightOpen,
  ChevronUp,
  ChevronDown,
  GripVertical,
} from 'lucide-react';

interface ResizableLayoutProps {
  leftPanel: React.ReactNode;
  centerPanel: React.ReactNode;
  rightPanel: React.ReactNode;
  bottomPanel?: React.ReactNode;
  defaultLeftWidth?: number;
  defaultRightWidth?: number;
  defaultBottomHeight?: number;
  leftCollapsed?: boolean;
  rightCollapsed?: boolean;
  bottomVisible?: boolean;
  onLeftCollapsedChange?: (collapsed: boolean) => void;
  onRightCollapsedChange?: (collapsed: boolean) => void;
  onBottomVisibleChange?: (visible: boolean) => void;
}

interface LayoutState {
  leftWidth: number;
  rightWidth: number;
  bottomHeight: number;
  leftCollapsed: boolean;
  rightCollapsed: boolean;
  bottomVisible: boolean;
}

const STORAGE_KEY = 'resizable-layout-state';
const LEFT_MIN = 150;
const RIGHT_MIN = 150;
const BOTTOM_MIN = 100;
const LEFT_DEFAULT = 250;
const RIGHT_DEFAULT = 400;
const BOTTOM_DEFAULT = 250;
const COLLAPSED_STRIP = 36;
const DIVIDER_WIDTH = 4;

function loadState(defaults: {
  leftWidth: number;
  rightWidth: number;
  bottomHeight: number;
  leftCollapsed: boolean;
  rightCollapsed: boolean;
  bottomVisible: boolean;
}): LayoutState {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) {
      const parsed = JSON.parse(raw);
      return {
        leftWidth: parsed.leftWidth ?? defaults.leftWidth,
        rightWidth: parsed.rightWidth ?? defaults.rightWidth,
        bottomHeight: parsed.bottomHeight ?? defaults.bottomHeight,
        leftCollapsed: parsed.leftCollapsed ?? defaults.leftCollapsed,
        rightCollapsed: parsed.rightCollapsed ?? defaults.rightCollapsed,
        bottomVisible: parsed.bottomVisible ?? defaults.bottomVisible,
      };
    }
  } catch {
    // ignore
  }
  return { ...defaults };
}

function saveState(state: LayoutState) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
  } catch {
    // ignore
  }
}

type Resizer = 'left' | 'right' | 'bottom' | null;

const ResizableLayout: React.FC<ResizableLayoutProps> = ({
  leftPanel,
  centerPanel,
  rightPanel,
  bottomPanel,
  defaultLeftWidth,
  defaultRightWidth,
  defaultBottomHeight,
  leftCollapsed: leftCollapsedProp,
  rightCollapsed: rightCollapsedProp,
  bottomVisible: bottomVisibleProp,
  onLeftCollapsedChange,
  onRightCollapsedChange,
  onBottomVisibleChange,
}) => {
  const [state, setState] = useState<LayoutState>(() =>
    loadState({
      leftWidth: defaultLeftWidth ?? LEFT_DEFAULT,
      rightWidth: defaultRightWidth ?? RIGHT_DEFAULT,
      bottomHeight: defaultBottomHeight ?? BOTTOM_DEFAULT,
      leftCollapsed: leftCollapsedProp ?? false,
      rightCollapsed: rightCollapsedProp ?? false,
      bottomVisible: bottomVisibleProp ?? true,
    }),
  );

  const activeResizer = useRef<Resizer>(null);
  const dragStart = useRef({ x: 0, y: 0, value: 0 });
  const containerRef = useRef<HTMLDivElement>(null);
  const stateRef = useRef(state);
  stateRef.current = state;

  const leftCollapsed = leftCollapsedProp ?? state.leftCollapsed;
  const rightCollapsed = rightCollapsedProp ?? state.rightCollapsed;
  const bottomVisible = bottomVisibleProp ?? state.bottomVisible;

  const updateState = useCallback(
    (patch: Partial<LayoutState>) => {
      setState((prev) => {
        const next = { ...prev, ...patch };
        saveState(next);
        return next;
      });
    },
    [],
  );

  const toggleLeft = useCallback(() => {
    const next = !leftCollapsed;
    updateState({ leftCollapsed: next });
    onLeftCollapsedChange?.(next);
  }, [leftCollapsed, updateState, onLeftCollapsedChange]);

  const toggleRight = useCallback(() => {
    const next = !rightCollapsed;
    updateState({ rightCollapsed: next });
    onRightCollapsedChange?.(next);
  }, [rightCollapsed, updateState, onRightCollapsedChange]);

  const toggleBottom = useCallback(() => {
    const next = !bottomVisible;
    updateState({ bottomVisible: next });
    onBottomVisibleChange?.(next);
  }, [bottomVisible, updateState, onBottomVisibleChange]);

  const handleDividerMouseDown = useCallback(
    (resizer: Resizer) => (e: React.MouseEvent) => {
      e.preventDefault();
      const s = stateRef.current;
      if (resizer === 'left') {
        dragStart.current = { x: e.clientX, y: 0, value: s.leftWidth };
      } else if (resizer === 'right') {
        dragStart.current = { x: e.clientX, y: 0, value: s.rightWidth };
      } else if (resizer === 'bottom') {
        dragStart.current = { x: 0, y: e.clientY, value: s.bottomHeight };
      }
      activeResizer.current = resizer;
    },
    [],
  );

  useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      if (!activeResizer.current || !containerRef.current) return;

      const containerRect = containerRef.current.getBoundingClientRect();

      if (activeResizer.current === 'left') {
        const delta = e.clientX - dragStart.current.x;
        const newWidth = Math.max(LEFT_MIN, dragStart.current.value + delta);
        updateState({ leftWidth: newWidth });
      } else if (activeResizer.current === 'right') {
        const delta = dragStart.current.x - e.clientX;
        const newWidth = Math.max(RIGHT_MIN, dragStart.current.value + delta);
        updateState({ rightWidth: newWidth });
      } else if (activeResizer.current === 'bottom') {
        const delta = dragStart.current.y - e.clientY;
        const maxBottom = containerRect.height * 0.7;
        const newHeight = Math.max(BOTTOM_MIN, Math.min(maxBottom, dragStart.current.value + delta));
        updateState({ bottomHeight: newHeight });
      }
    };

    const handleMouseUp = () => {
      if (activeResizer.current) {
        activeResizer.current = null;
        document.body.style.cursor = '';
        document.body.style.userSelect = '';
      }
    };

    if (activeResizer.current) {
      const cursor =
        activeResizer.current === 'bottom' ? 'row-resize' : 'col-resize';
      document.body.style.cursor = cursor;
      document.body.style.userSelect = 'none';
      document.addEventListener('mousemove', handleMouseMove);
      document.addEventListener('mouseup', handleMouseUp);
    }

    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };
  }, [updateState]);

  const isDragging = activeResizer.current !== null;

  const collapsedStrip = (
    side: 'left' | 'right',
    onClick: () => void,
    collapsed: boolean,
  ) => {
    if (!collapsed) return null;
    const isLeft = side === 'left';
    return (
      <div
        className="flex-shrink-0 flex flex-col items-center justify-center cursor-pointer z-10"
        style={{
          width: COLLAPSED_STRIP,
          background: '#1a1a2e',
          borderColor: '#2d2d44',
          borderRightWidth: isLeft ? 1 : 0,
          borderLeftWidth: isLeft ? 0 : 1,
          borderStyle: 'solid',
        }}
        onClick={onClick}
      >
        <button
          className="p-1 rounded hover:bg-[#2d2d44] transition-colors text-[#888] hover:text-[#d97706]"
          title={isLeft ? 'Expand left panel' : 'Expand right panel'}
        >
          {isLeft ? <PanelLeftOpen size={16} /> : <PanelRightOpen size={16} />}
        </button>
      </div>
    );
  };

  const renderDivider = (
    type: 'horizontal' | 'vertical',
    resizer: Resizer,
    visible = true,
  ) => {
    if (!visible) return null;
    const isActive = isDragging && activeResizer.current === resizer;
    const isVertical = type === 'vertical';

    return (
      <div
        onMouseDown={handleDividerMouseDown(resizer)}
        className={`flex-shrink-0 group relative z-20 ${
          isVertical
            ? 'w-[4px] cursor-col-resize'
            : 'h-[4px] cursor-row-resize'
        }`}
        style={{ background: isActive ? '#d97706' : '#2d2d44' }}
      >
        <div
          className={`absolute inset-0 opacity-0 group-hover:opacity-100 transition-opacity ${
            isActive ? 'opacity-100' : ''
          }`}
          style={{ background: '#d97706' }}
        />
        <div
          className={`absolute ${
            isVertical
              ? 'inset-y-0 left-1/2 -translate-x-1/2 w-[16px]'
              : 'inset-x-0 top-1/2 -translate-y-1/2 h-[16px]'
          } flex items-center justify-center`}
        >
          <GripVertical
            size={12}
            className={`${isVertical ? '' : 'rotate-90'} ${
              isActive ? 'text-[#d97706]' : 'text-transparent group-hover:text-[#666]'
            } transition-colors`}
          />
        </div>
      </div>
    );
  };

  const leftWidth = leftCollapsed ? 0 : state.leftWidth;
  const rightWidth = rightCollapsed ? 0 : state.rightWidth;
  const bottomHeight = bottomVisible ? state.bottomHeight : 0;

  return (
    <div
      ref={containerRef}
      className="w-full h-full flex flex-col overflow-hidden"
      style={{ background: '#1a1a2e' }}
    >
      <div className="flex flex-1 overflow-hidden min-h-0">
        {collapsedStrip('left', toggleLeft, leftCollapsed)}

        <AnimatePresence initial={false}>
          {!leftCollapsed && (
            <motion.div
              key="left-panel"
              initial={{ width: 0, opacity: 0 }}
              animate={{ width: leftWidth, opacity: 1 }}
              exit={{ width: 0, opacity: 0 }}
              transition={{ duration: 0.2, ease: 'easeInOut' }}
              className="flex-shrink-0 overflow-hidden relative flex flex-col"
              style={{ borderRight: `1px solid #2d2d44` }}
            >
              <div className="absolute top-2 right-2 z-10">
                <button
                  onClick={toggleLeft}
                  className="p-1 rounded hover:bg-[#2d2d44] transition-colors text-[#888] hover:text-[#d97706]"
                  title="Collapse left panel"
                >
                  <PanelLeftClose size={14} />
                </button>
              </div>
              <div className="flex-1 overflow-auto">{leftPanel}</div>
            </motion.div>
          )}
        </AnimatePresence>

        {renderDivider('vertical', 'left', !leftCollapsed)}

        <div className="flex-1 overflow-auto min-w-0">{centerPanel}</div>

        {renderDivider('vertical', 'right', !rightCollapsed)}

        <AnimatePresence initial={false}>
          {!rightCollapsed && (
            <motion.div
              key="right-panel"
              initial={{ width: 0, opacity: 0 }}
              animate={{ width: rightWidth, opacity: 1 }}
              exit={{ width: 0, opacity: 0 }}
              transition={{ duration: 0.2, ease: 'easeInOut' }}
              className="flex-shrink-0 overflow-hidden relative flex flex-col"
              style={{ borderLeft: `1px solid #2d2d44` }}
            >
              <div className="absolute top-2 left-2 z-10">
                <button
                  onClick={toggleRight}
                  className="p-1 rounded hover:bg-[#2d2d44] transition-colors text-[#888] hover:text-[#d97706]"
                  title="Collapse right panel"
                >
                  <PanelRightClose size={14} />
                </button>
              </div>
              <div className="flex-1 overflow-auto">{rightPanel}</div>
            </motion.div>
          )}
        </AnimatePresence>

        {collapsedStrip('right', toggleRight, rightCollapsed)}
      </div>

      {bottomPanel && (
        <>
          {renderDivider('horizontal', 'bottom', bottomVisible)}

          <AnimatePresence initial={false}>
            {bottomVisible && (
              <motion.div
                key="bottom-panel"
                initial={{ height: 0, opacity: 0 }}
                animate={{ height: bottomHeight, opacity: 1 }}
                exit={{ height: 0, opacity: 0 }}
                transition={{ duration: 0.2, ease: 'easeInOut' }}
                className="flex-shrink-0 overflow-hidden flex flex-col"
                style={{ borderTop: `1px solid #2d2d44` }}
              >
                <div className="flex items-center justify-between px-3 h-8 flex-shrink-0" style={{ background: '#1a1a2e' }}>
                  <span className="text-xs text-[#888] font-medium">Terminal</span>
                  <button
                    onClick={toggleBottom}
                    className="p-1 rounded hover:bg-[#2d2d44] transition-colors text-[#888] hover:text-[#d97706]"
                    title="Hide terminal"
                  >
                    <ChevronDown size={14} />
                  </button>
                </div>
                <div className="flex-1 overflow-auto">{bottomPanel}</div>
              </motion.div>
            )}
          </AnimatePresence>

          {!bottomVisible && (
            <div
              className="flex-shrink-0 flex items-center justify-center cursor-pointer h-7 z-10"
              style={{
                background: '#1a1a2e',
                borderTop: `1px solid #2d2d44`,
              }}
              onClick={toggleBottom}
            >
              <button
                className="p-0.5 rounded hover:bg-[#2d2d44] transition-colors text-[#888] hover:text-[#d97706]"
                title="Show terminal"
              >
                <ChevronUp size={14} />
              </button>
            </div>
          )}
        </>
      )}
    </div>
  );
};

export default ResizableLayout;