import React from 'react';
import sidebarToggleImg from '../assets/icons/sidebar-toggle.png';
import newChatImg from '../assets/icons/new_chat.png';
import chatsImg from '../assets/icons/chats.png';
import projectsImg from '../assets/icons/projects.png';
import artifactsImg from '../assets/icons/artifacts.png';
import codeImg from '../assets/icons/code.png';

// Sidebar Toggle
export const IconSidebarToggle = ({ size = 20, className = "" }: { size?: number, className?: string }) => (
  <img src={sidebarToggleImg} width={size} height={size} className={className} alt="Sidebar Toggle" />
);

// New Chat button icon
export const IconPlusCircle = ({ size = 20, className = "" }: { size?: number, className?: string }) => (
  <img src={newChatImg} width={size} height={size} className={className} alt="New Chat" />
);

// Generic Plus - Keeping SVG for generic use unless requested
export const IconPlus = ({ size = 20, className = "" }: { size?: number, className?: string }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className={className}>
    <line x1="12" y1="5" x2="12" y2="19" />
    <line x1="5" y1="12" x2="19" y2="12" />
  </svg>
);

// Research icon - magnifying glass with zigzag trend line inside
export const IconResearch = ({ size = 20, className = "" }: { size?: number, className?: string }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className={className}>
    <circle cx="10.5" cy="10.5" r="7.5" />
    <line x1="21" y1="21" x2="16" y2="16" />
    <polyline points="6.5 12 9 9 11 11 14.5 7.5" />
  </svg>
);

// Web Search icon - globe with meridian curves. Drawn inline so `currentColor` lets
// callers tint to the blue accent when enabled. Matches the user-provided icon shape
// (globe with vertical + horizontal sweep curves) while staying a proper stroked SVG.
export const IconWebSearch = ({ size = 20, className = "" }: { size?: number, className?: string }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" className={className}>
    <circle cx="12" cy="12" r="10" />
    <path d="M2 12h20" />
    <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
  </svg>
);

// Chats
export const IconChatBubble = ({ size = 20, className = "" }: { size?: number, className?: string }) => (
  <img src={chatsImg} width={size} height={size} className={className} alt="Chats" />
);

// Projects — PNG icon. The source is dark-on-transparent. The default render gives
// the icon no special dark-mode treatment — callers decide. The sidebar uses a plain
// `dark:invert` (matching other nav icons), while the plus-menu callers add an
// inline filter to hit the warm light gray ~#ABA499 tone of stroke-based icons there.
export const IconProjects = ({ size = 20, className = "" }: { size?: number, className?: string }) => (
  <img src={projectsImg} width={size} height={size} className={className} alt="Projects" />
);

// Artifacts
export const IconArtifacts = ({ size = 20, className = "" }: { size?: number, className?: string }) => (
  <img src={artifactsImg} width={size} height={size} className={className} alt="Artifacts" />
);

// Artifacts Exact (Sidebar)
export const IconArtifactsExact = ({ size = 20, className = "" }: { size?: number, className?: string }) => (
  <img src={artifactsImg} width={size} height={size} className={className} alt="Artifacts" />
);


// Code
export const IconCode = ({ size = 20, className = "" }: { size?: number, className?: string }) => (
  <img src={codeImg} width={size} height={size} className={className} alt="Code" />
);

export const IconNewChat = ({ size = 20, className = "" }: { size?: number, className?: string }) => (
  <img src={newChatImg} width={size} height={size} className={className} alt="New Chat" />
);

export const IconVoice = ({ size = 20, className = "" }: { size?: number, className?: string }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className={className}>
    <line x1="12" y1="6" x2="12" y2="18" />
    <line x1="8" y1="9" x2="8" y2="15" />
    <line x1="16" y1="9" x2="16" y2="15" />
    <line x1="4" y1="11" x2="4" y2="13" />
    <line x1="20" y1="11" x2="20" y2="13" />
  </svg>
);

export const IconDotsHorizontal = ({ size = 20, className = "" }: { size?: number, className?: string }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className={className}>
    <circle cx="12" cy="12" r="1" />
    <circle cx="19" cy="12" r="1" />
    <circle cx="5" cy="12" r="1" />
  </svg>
);

export const IconStarOutline = ({ size = 20, className = "" }: { size?: number, className?: string }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" className={className}>
    <path d="M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z" />
  </svg>
);

export const IconPencil = ({ size = 20, className = "" }: { size?: number, className?: string }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" className={className}>
    <path d="M17 3a2.828 2.828 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z" />
  </svg>
);

export const IconTrash = ({ size = 20, className = "" }: { size?: number, className?: string }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" className={className}>
    <path d="M3 6h18" />
    <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6" />
    <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2" />
  </svg>
);