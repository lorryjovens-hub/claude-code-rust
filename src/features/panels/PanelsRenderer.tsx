import React from 'react';
import ResearchPanel from '../../components/ResearchPanel';
import McpManagementPanel from '../../components/McpManagementPanel';
import H5AccessPanel from '../../components/H5AccessPanel';
import TerminalPanel from '../../components/TerminalPanel';
import ComputerUsePanel from '../../components/ComputerUsePanel';
import SlashCommandPalette from '../../components/SlashCommandPalette';

interface PanelsRendererProps {
  openedResearchMsgId: string | null;
  messages: any[];
  setOpenedResearchMsgId: (id: string | null) => void;
  showMcpPanel: boolean;
  setShowMcpPanel: (show: boolean) => void;
  showH5Panel: boolean;
  setShowH5Panel: (show: boolean) => void;
  showTerminalPanel: boolean;
  setShowTerminalPanel: (show: boolean) => void;
  showComputerUsePanel: boolean;
  setShowComputerUsePanel: (show: boolean) => void;
  showSlashPalette: boolean;
  setShowSlashPalette: (show: boolean) => void;
  slashPaletteInput: string;
  setSlashPaletteInput: (v: string) => void;
  setInputText: (v: string) => void;
  inputRef: React.RefObject<HTMLTextAreaElement | null>;
  activeId: string | null;
}

export default function PanelsRenderer({
  openedResearchMsgId, messages, setOpenedResearchMsgId,
  showMcpPanel, setShowMcpPanel,
  showH5Panel, setShowH5Panel,
  showTerminalPanel, setShowTerminalPanel,
  showComputerUsePanel, setShowComputerUsePanel,
  showSlashPalette, setShowSlashPalette,
  slashPaletteInput, setSlashPaletteInput,
  setInputText, inputRef, activeId,
}: PanelsRendererProps) {
  return (
    <>
      {/* Research panel — fixed right-side drawer */}
      {openedResearchMsgId && (() => {
        const liveMsg = messages.find((m: any) => m.id === openedResearchMsgId);
        if (!liveMsg || !liveMsg.research) return null;
        return (
          <>
            <div
              className="fixed inset-0 z-[60] bg-black/20"
              onClick={() => setOpenedResearchMsgId(null)}
            />
            <div className="fixed top-0 right-0 bottom-0 w-[440px] z-[61] bg-claude-bg border-l border-claude-border shadow-2xl flex flex-col">
              <ResearchPanel research={liveMsg.research as any} onClose={() => setOpenedResearchMsgId(null)} />
            </div>
          </>
        );
      })()}

      {/* Slash Command Palette */}
      <SlashCommandPalette
        isOpen={showSlashPalette}
        onClose={() => { setShowSlashPalette(false); setSlashPaletteInput(''); }}
        onSelect={(cmd) => {
          setShowSlashPalette(false);
          setInputText(cmd + ' ');
          inputRef.current?.focus();
        }}
        inputValue={slashPaletteInput}
      />

      {/* MCP Management Panel */}
      {showMcpPanel && (
        <McpManagementPanel onClose={() => setShowMcpPanel(false)} />
      )}

      {/* H5 Access Panel */}
      {showH5Panel && activeId && (
        <H5AccessPanel
          conversationId={activeId}
          onOpenChange={(open) => { if (!open) setShowH5Panel(false); }}
        />
      )}

      {/* Terminal Panel */}
      {showTerminalPanel && (
        <div
          className="fixed z-50 border border-claude-border bg-[#1e1e2e] flex flex-col shadow-2xl transition-all duration-300 ease-in-out"
          style={{
            top: '88px', right: '16px',
            width: 'min(800px, calc(100vw - 340px))',
            height: 'min(500px, calc(100vh - 120px))',
            borderRadius: '12px', overflow: 'hidden',
          }}
        >
          <TerminalPanel onClose={() => setShowTerminalPanel(false)} />
        </div>
      )}

      {/* Computer Use Panel */}
      {showComputerUsePanel && (
        <ComputerUsePanel onClose={() => setShowComputerUsePanel(false)} />
      )}
    </>
  );
}
