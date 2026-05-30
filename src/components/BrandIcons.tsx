import React from 'react';

// Telegram Official Icon - Paper plane with brand gradient
export const IconTelegram = ({ size = 24, className = "" }: { size?: number, className?: string }) => (
  <svg width={size} height={size} viewBox="0 0 48 48" fill="none" xmlns="http://www.w3.org/2000/svg" className={className}>
    <defs>
      <linearGradient id="tg-grad" x1="24" y1="0" x2="24" y2="48" gradientUnits="userSpaceOnUse">
        <stop offset="0%" stopColor="#2AABEE"/>
        <stop offset="100%" stopColor="#229ED9"/>
      </linearGradient>
    </defs>
    <circle cx="24" cy="24" r="24" fill="url(#tg-grad)"/>
    <path d="M11.5 24.5c6.5-2.8 10.8-4.7 13-5.5 6.2-2.6 7.5-3 8.3-3 .2 0 .5 0 .7.2.2.2.2.4.2.6 0 .4-.1.8-.3 1.3-.9 3.2-2.9 10.9-4.1 14.5-.5 1.6-1 2.2-1.5 2.2-.3 0-.6-.1-1-.3-.5-.3-2.2-1.4-3.8-2.6-1.9-1.4-3.1-2.3-3.5-2.6-.4-.4-.1-.6.3-.9.2-.1.6-.4 1-.7.4-.3.8-.5 1.1-.8.3-.3.6-.5.9-.8.3-.3.5-.5.8-.8.5-.5.9-1 1.1-1.5.1-.3 0-.5-.1-.7-.1-.1-.3-.2-.5-.2h-.3c-.5 0-1.2.2-2.2.6-2.9 1.2-5.1 2.1-6.5 2.7-1.5.6-2.7.9-3.5.9-.4 0-.7-.1-1-.2-.3-.2-.4-.4-.5-.7 0-.2-.1-.4 0-.6z" fill="white"/>
  </svg>
);

// WeChat Official Icon - Dual speech bubbles with brand green
export const IconWeChat = ({ size = 24, className = "" }: { size?: number, className?: string }) => (
  <svg width={size} height={size} viewBox="0 0 48 48" fill="none" xmlns="http://www.w3.org/2000/svg" className={className}>
    <defs>
      <linearGradient id="wx-grad" x1="24" y1="0" x2="24" y2="48" gradientUnits="userSpaceOnUse">
        <stop offset="0%" stopColor="#07C160"/>
        <stop offset="100%" stopColor="#05A350"/>
      </linearGradient>
    </defs>
    <circle cx="24" cy="24" r="24" fill="url(#wx-grad)"/>
    <g fill="white">
      <path d="M19.5 14c-4.1 0-7.5 2.8-7.5 6.3 0 2 1.1 3.8 2.8 5-.1.5-.4 1.4-.9 2.1 0 0-.2.3 0 .5.2.2.5 0 .5 0 .9-.6 2-1.1 2.6-1.4.8.2 1.7.4 2.6.4.4 0 .8 0 1.2-.1-.1-.5-.2-1-.2-1.5 0-3.9 3.7-7.1 8.2-7.1.3 0 .5 0 .8.1C28.8 16.4 24.5 14 19.5 14zM16 19c-.6 0-1-.4-1-1s.4-1 1-1 1 .4 1 1-.4 1-1 1zm6 0c-.6 0-1-.4-1-1s.4-1 1-1 1 .4 1 1-.4 1-1 1z"/>
      <path d="M35.5 23.8c0-3.3-3.2-6-7.1-6s-7.1 2.7-7.1 6c0 3.3 3.2 6 7.1 6 .7 0 1.4-.1 2.1-.3.5.2 1.3.6 2 .1 0 0 .3.2.4 0 .2-.2 0-.4 0-.4-.4-.6-.7-1.3-.8-1.8 1.4-1.1 2.4-2.6 2.4-4.4zm-8.8-.8c-.5 0-1-.4-1-1s.4-1 1-1 1 .4 1 1-.4 1-1 1zm5 0c-.5 0-1-.4-1-1s.4-1 1-1 1 .4 1 1-.4 1-1 1z"/>
    </g>
  </svg>
);

// Feishu (Lark) Official Icon - Stylized bird with brand blue
export const IconFeishu = ({ size = 24, className = "" }: { size?: number, className?: string }) => (
  <svg width={size} height={size} viewBox="0 0 48 48" fill="none" xmlns="http://www.w3.org/2000/svg" className={className}>
    <defs>
      <linearGradient id="fs-grad" x1="24" y1="0" x2="24" y2="48" gradientUnits="userSpaceOnUse">
        <stop offset="0%" stopColor="#3370FF"/>
        <stop offset="100%" stopColor="#1E4FD9"/>
      </linearGradient>
    </defs>
    <circle cx="24" cy="24" r="24" fill="url(#fs-grad)"/>
    <path d="M28.5 12c-1.5 0-3 .5-4 1.5l-8.5 8c-1 1-1.5 2.5-1.5 4s.5 3 1.5 4l4 4c1 1 2.5 1.5 4 1.5s3-.5 4-1.5l8.5-8c1-1 1.5-2.5 1.5-4s-.5-3-1.5-4l-4-4c-1-1-2.5-1.5-4-1.5z" fill="white" opacity="0.95"/>
    <path d="M24.5 18.5l-4.5 4.5c-.5.5-.5 1.5 0 2l2 2c.5.5 1.5.5 2 0l4.5-4.5c.5-.5.5-1.5 0-2l-2-2c-.5-.5-1.5-.5-2 0z" fill="#3370FF"/>
  </svg>
);

// DingTalk Official Icon - Lightning bolt with brand blue
export const IconDingTalk = ({ size = 24, className = "" }: { size?: number, className?: string }) => (
  <svg width={size} height={size} viewBox="0 0 48 48" fill="none" xmlns="http://www.w3.org/2000/svg" className={className}>
    <defs>
      <linearGradient id="dt-grad" x1="24" y1="0" x2="24" y2="48" gradientUnits="userSpaceOnUse">
        <stop offset="0%" stopColor="#0089FF"/>
        <stop offset="100%" stopColor="#0066CC"/>
      </linearGradient>
    </defs>
    <circle cx="24" cy="24" r="24" fill="url(#dt-grad)"/>
    <path d="M26.5 12l-8 14h4l-2 10 10-14h-4l4-10h-4z" fill="white" stroke="white" strokeWidth="1.5" strokeLinejoin="round"/>
  </svg>
);

// Generic IM/Message icon for sidebar
export const IconIM = ({ size = 24, className = "" }: { size?: number, className?: string }) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" className={className}>
    <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>
    <path d="M8 9h8"/>
    <path d="M8 13h5"/>
  </svg>
);

// Platform icon wrapper with consistent sizing
export const PlatformIcon = ({ platform, size = 24, className = "" }: { platform: 'telegram' | 'wechat' | 'feishu' | 'dingtalk' | 'lark_bridge', size?: number, className?: string }) => {
  switch (platform) {
    case 'telegram': return <IconTelegram size={size} className={className} />;
    case 'wechat': return <IconWeChat size={size} className={className} />;
    case 'feishu':
    case 'lark_bridge': return <IconFeishu size={size} className={className} />;
    case 'dingtalk': return <IconDingTalk size={size} className={className} />;
    default: return <IconIM size={size} className={className} />;
  }
};
