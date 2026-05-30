import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import ClaudeLogo from '../components/ClaudeLogo';

// Mock canvas API for jsdom
const mockCanvasContext = {
  clearRect: vi.fn(),
  fillRect: vi.fn(),
  beginPath: vi.fn(),
  moveTo: vi.fn(),
  lineTo: vi.fn(),
  bezierCurveTo: vi.fn(),
  closePath: vi.fn(),
  fill: vi.fn(),
  stroke: vi.fn(),
  save: vi.fn(),
  restore: vi.fn(),
  translate: vi.fn(),
  scale: vi.fn(),
  rotate: vi.fn(),
  setTransform: vi.fn(),
  createRadialGradient: vi.fn().mockReturnValue({
    addColorStop: vi.fn(),
  }),
  createLinearGradient: vi.fn().mockReturnValue({
    addColorStop: vi.fn(),
  }),
  arc: vi.fn(),
  fillStyle: '',
  strokeStyle: '',
  lineWidth: 1,
  shadowColor: '',
  shadowBlur: 0,
  globalAlpha: 1,
};

vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockImplementation(() => mockCanvasContext as any);

describe('ClaudeLogo', () => {
  it('should render canvas element', () => {
    render(<ClaudeLogo />);
    const canvas = document.querySelector('canvas');
    expect(canvas).toBeInTheDocument();
  });

  it('should accept className prop', () => {
    render(<ClaudeLogo className="test-class" />);
    const container = document.querySelector('div.test-class');
    expect(container).toBeInTheDocument();
  });

  it('should render with default props', () => {
    render(<ClaudeLogo />);
    const canvas = document.querySelector('canvas');
    expect(canvas).toBeInTheDocument();
  });

  it('should accept color prop', () => {
    render(<ClaudeLogo color="#FF0000" />);
    const canvas = document.querySelector('canvas');
    expect(canvas).toBeInTheDocument();
  });
});
