import React from 'react';

export const StarIcon = ({ className = "" }: { className?: string }) => {
  return (
    <svg
      viewBox="0 0 100 100"
      fill="currentColor"
      className={className}
      xmlns="http://www.w3.org/2000/svg"
    >
      {/* 7-spoke spark approximation */}
      <path d="M43.7,13.6 C43.5,10.6 47.7,10.1 48.7,13.1 L53.7,29.9 C54.3,31.8 56.6,32.7 58.3,31.7 L72.7,23.4 C75.3,21.9 77.8,25.2 75.8,27.5 L64.8,39.9 C63.5,41.4 63.8,43.7 65.4,44.8 L80.4,55.1 C82.8,56.8 81.3,60.6 78.4,60.6 L62.3,60.6 C60.3,60.6 58.7,62.0 58.3,63.9 L54.7,79.4 C53.9,82.3 49.7,82.3 48.9,79.4 L45.3,63.9 C44.9,62.0 43.3,60.6 41.3,60.6 L25.2,60.6 C22.3,60.6 20.8,56.8 23.2,55.1 L38.2,44.8 C39.8,43.7 40.1,41.4 38.8,39.9 L27.8,27.5 C25.8,25.2 28.3,21.9 30.9,23.4 L45.3,31.7 C47.0,32.7 49.3,31.8 49.9,29.9 L54.9,13.1 C55.9,10.1 60.1,10.6 59.9,13.6 L56.2,28.6 C55.8,30.5 54.2,31.9 52.2,31.9 L36.1,31.9" opacity="0" />

      {/* Hand-drawn style approximation paths */}
      <path d="M48,15 C46,12 52,12 51,15 L55,35 L65,25 C67,23 69,27 67,29 L58,40 L75,45 C78,46 76,50 73,50 L58,52 L62,68 C63,71 59,73 57,71 L48,55 L38,70 C36,72 32,70 33,67 L38,50 L22,48 C19,48 19,44 22,43 L38,40 L30,28 C28,26 31,23 33,25 L45,35 L48,15 Z" fill="currentColor" opacity="0.1" />

      {/* Actual defined strokes for 7 rays */}
      <rect x="46" y="12" width="8" height="38" rx="4" transform="rotate(-15 50 50)" />
      <rect x="46" y="12" width="8" height="38" rx="4" transform="rotate(40 50 50)" />
      <rect x="46" y="12" width="8" height="38" rx="4" transform="rotate(95 50 50)" />
      <rect x="46" y="12" width="8" height="38" rx="4" transform="rotate(145 50 50)" />
      <rect x="46" y="12" width="8" height="38" rx="4" transform="rotate(195 50 50)" />
      <rect x="46" y="12" width="8" height="38" rx="4" transform="rotate(250 50 50)" />
      <rect x="46" y="12" width="8" height="38" rx="4" transform="rotate(305 50 50)" />
    </svg>
  );
};