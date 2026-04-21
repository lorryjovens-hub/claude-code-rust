import React, { useEffect, useRef } from 'react';

// 核心数据 - 来自 claude logo.txt
const CORE_RADIUS = 27.2;
const ROOT_WIDTH_SCALE = 1.1;
const TIP_WIDTH_SCALE = 1.3;

const INITIAL_TENTACLES = [
  { id: 1, angle: -176.8, length: 82.0, baseW: 10.0, tipW: 11.2 },
  { id: 2, angle: -144.1, length: 83.6, baseW: 16.5, tipW: 19.1 },
  { id: 3, angle: -117.6, length: 89.2, baseW: 17.9, tipW: 23.6 },
  { id: 4, angle: -82.1,  length: 73.2, baseW: 13.2, tipW: 16.6 },
  { id: 5, angle: -50.8,  length: 73.4, baseW: 26.5, tipW: 21.5 },
  { id: 6, angle: -10.9,  length: 67.7, baseW: 15.2, tipW: 13.7 },
  { id: 7, angle: 9.1,    length: 69.3, baseW: 9.8,  tipW: 18.2 },
  { id: 8, angle: 40.6,   length: 71.1, baseW: 13.4, tipW: 8.9 },
  { id: 9, angle: 56.2,   length: 68.2, baseW: 18.0, tipW: 15.3 },
  { id: 10, angle: 98.5,  length: 73.8, baseW: 9.8,  tipW: 15.7 },
  { id: 11, angle: 128.1, length: 77.6, baseW: 13.5, tipW: 11.8 },
  { id: 12, angle: 148.1, length: 75.0, baseW: 13.6, tipW: 14.4 }
].map((tentacle) => ({
  ...tentacle,
  baseW: tentacle.baseW * ROOT_WIDTH_SCALE,
  tipW: tentacle.tipW * TIP_WIDTH_SCALE,
}));

interface ClaudeLogoProps {
  className?: string;
  style?: React.CSSProperties;
  onClick?: () => void;
  autoAnimate?: boolean; // 对应 thinking 模式
  breathe?: boolean;     // 对应 waiting 模式
  color?: string;
  maxScale?: number;     // cap the drawing scale (e.g. 0.17 to keep logo small in a large canvas)
}

const ClaudeLogo: React.FC<ClaudeLogoProps> = ({ className = '', style, onClick, autoAnimate = false, breathe = false, color = '#D97757', maxScale }) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const requestRef = useRef<number | null>(null);
  
  // 实例状态
  const tentaclesRef = useRef(INITIAL_TENTACLES.map(t => ({
    ...t,
    animLength: t.length,
    animBaseW: t.baseW,
    animTipW: t.tipW,
    jL: 0, jB: 0, jT: 0
  })));
  
  const mouseRef = useRef({ x: -9999, y: -9999, present: false });
  const lastJitterTimeRef = useRef(0);
  const stateRef = useRef({
    scale: 0.8, // 将根据容器大小动态更新
    smoothness: 0,
    fillet: 6.0,
    color: color,
    mode: 'idle',
    maxScale: maxScale,
  });

  // 监听颜色变化
  useEffect(() => {
    stateRef.current.color = color;
  }, [color]);

  // 更新模式
  useEffect(() => {
    if (breathe) {
      stateRef.current.mode = 'waiting';
    } else if (autoAnimate) {
      stateRef.current.mode = 'thinking';
    } else {
      stateRef.current.mode = 'idle';
    }
  }, [breathe, autoAnimate]);

  useEffect(() => {
    const canvas = canvasRef.current;
    const container = containerRef.current;
    if (!canvas || !container) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const tentacles = tentaclesRef.current;
    const state = stateRef.current;
    const mouse = mouseRef.current;

    const JITTER_FPS = 3;
    const jitterInterval = 1000 / JITTER_FPS;

    // 调整大小
    const handleResize = () => {
      const rect = container.getBoundingClientRect();
      const dpr = window.devicePixelRatio || 1;
      const width = Math.max(1, Math.round(rect.width));
      const height = Math.max(1, Math.round(rect.height));
      
      canvas.width = Math.max(1, Math.round(width * dpr));
      canvas.height = Math.max(1, Math.round(height * dpr));
      
      // 计算缩放比例
      // 参考尺寸: 240 (大约是 tentacles 展开后的直径)
      const minDim = Math.min(width, height);
      state.scale = (minDim / 240) * 0.85;
      if (state.maxScale !== undefined && state.scale > state.maxScale) state.scale = state.maxScale;
      
      ctx.setTransform(1, 0, 0, 1, 0, 0);
      ctx.scale(dpr, dpr);
    };

    // 鼠标事件
    const handleMouseMove = (e: MouseEvent) => {
      const rect = canvas.getBoundingClientRect();
      mouse.x = e.clientX - rect.left;
      mouse.y = e.clientY - rect.top;
      mouse.present = true;
    };

    const handleMouseLeave = () => {
      mouse.present = false;
      mouse.x = -9999;
      mouse.y = -9999;
    };

    // 动画循环
    const animate = (timestamp: number) => {
      requestRef.current = requestAnimationFrame(animate);
      
      const rect = container.getBoundingClientRect();
      const cx = rect.width / 2;
      const cy = rect.height / 2;

      // 物理更新
      let shouldUpdateJitter = false;
      if (timestamp - lastJitterTimeRef.current >= jitterInterval) {
        shouldUpdateJitter = true;
        lastJitterTimeRef.current = timestamp;
      }

      tentacles.forEach(t => {
        const rad = t.angle * Math.PI / 180;
        let targetLength = t.length;
        let targetBaseW = t.baseW;
        let targetTipW = t.tipW;

        if (state.mode === 'idle') {
            const MAX_DIST = 360; 
            // 计算相对于 Logo 内部坐标系的鼠标位置
            // 这里我们保持 claude logo.txt 的逻辑，即除以 scale
            // 这样无论 Logo 多大，交互范围相对于 Logo 自身的比例是恒定的
            let localMouseX = (mouse.x - cx) / state.scale;
            let localMouseY = (mouse.y - cy) / state.scale;
            
            const GRID_SIZE = 40;
            localMouseX = Math.round(localMouseX / GRID_SIZE) * GRID_SIZE;
            localMouseY = Math.round(localMouseY / GRID_SIZE) * GRID_SIZE;
            
            let d_m = Math.hypot(localMouseX, localMouseY);
            let E = 0; let P = 1; let centerBlend = 0;
            
            if (mouse.present && d_m < MAX_DIST) {
                E = d_m < 200 ? 1 : 1 - Math.pow(1 - (1 - (d_m - 200) / 160), 2);
                if (d_m > 200) P = 1; else if (d_m < 60) P = -1; else P = Math.sin(((d_m - 60) / 140 * 2 - 1) * Math.PI / 2);
                if (d_m < 50) centerBlend = 1 - d_m / 50;
                E = Math.round(E * 5) / 5; P = Math.round(P * 5) / 5;
                let alignment = Math.round((Math.cos(rad - Math.atan2(localMouseY, localMouseX)) * (1 - centerBlend) + centerBlend) * 4) / 4;
                let wS = (P + 1) / 2; let wQ = (1 - P) / 2;
                if (alignment > 0) {
                    const str = Math.pow(alignment, 1.5);
                    targetLength *= (1 + E * (wS * 0.4 + wQ * -0.45) * str);
                    targetBaseW *= (1 + E * (wS * 0.3 + wQ * 0.6) * str);
                    targetTipW *= (1 + E * (wS * 1.2 + wQ * -0.15) * str);
                } else {
                    const str = Math.pow(Math.abs(alignment), 2);
                    targetLength *= (1 + E * (wS * -0.5 + wQ * -0.2) * str);
                    targetBaseW *= (1 + E * (wS * -0.35 + wQ * 0.2) * str);
                    targetTipW *= (1 + E * (wS * -0.35 + wQ * -0.1) * str);
                }
                if (shouldUpdateJitter) {
                    t.jL = Math.round((Math.random() - 0.5) * 4) / 2;
                    t.jB = Math.round((Math.random() - 0.5) * 4) / 2;
                    t.jT = Math.round((Math.random() - 0.5) * 4) / 2;
                }
                const amt = 0.08 * E;
                targetLength *= (1 + t.jL * amt);
                targetBaseW *= (1 + t.jB * amt * 0.5);
                targetTipW *= (1 + t.jT * amt);
            }
            t.animLength = targetLength;
            t.animBaseW = targetBaseW;
            t.animTipW = targetTipW;
        } else if (state.mode === 'waiting') {
            const t_shrink = 250; const t_expand = 550; const t_pause = 250;
            const total_cycle = t_shrink + t_expand + t_pause;
            const progress = timestamp % total_cycle;
            let raw_k;
            if (progress < t_shrink) raw_k = Math.cos((progress / t_shrink) * Math.PI / 2);
            else if (progress < t_shrink + t_expand) raw_k = Math.sin(((progress - t_shrink) / t_expand) * Math.PI / 2);
            else raw_k = 1;
            const k = Math.round(raw_k * 10) / 10;
            targetLength = t.length * k;
            t.animLength += (targetLength - t.animLength) * 0.45;
            t.animBaseW = t.baseW; t.animTipW = t.tipW;
        } else if (state.mode === 'thinking') {
            const rotSpeed = timestamp / 140; 
            let spiralPhase = (rad - rotSpeed) % (Math.PI * 2);
            if (spiralPhase < 0) spiralPhase += Math.PI * 2;
            let normPhase = spiralPhase / (Math.PI * 2); 
            let smoothK;
            if (normPhase < 0.85) {
                smoothK = 0.45 + 0.85 * (normPhase / 0.85);
            } else {
                let p = (normPhase - 0.85) / 0.15;
                smoothK = 1.3 - 0.85 * p;
            }
            const STEPS = 12;
            const k = Math.round(smoothK * STEPS) / STEPS;
            targetLength = t.length * k;
            const w_k = 0.9 + 0.25 * (k - 0.45);
            targetBaseW = t.baseW * w_k;
            targetTipW = t.tipW * w_k;
            t.animLength += (targetLength - t.animLength) * 0.6;
            t.animBaseW += (targetBaseW - t.animBaseW) * 0.6;
            t.animTipW += (targetTipW - t.animTipW) * 0.6;
            if (shouldUpdateJitter) {
                t.jL = (Math.random() - 0.5) * 0.05;
            }
            t.animLength *= (1 + t.jL);
        }
      });

      // 绘制
      ctx.clearRect(0, 0, rect.width, rect.height);
      ctx.save();
      ctx.translate(cx, cy);
      ctx.scale(state.scale, state.scale);
      ctx.fillStyle = state.color;
      ctx.strokeStyle = state.color;
      ctx.lineWidth = 0; ctx.lineJoin = 'round'; ctx.lineCap = 'round';

      tentacles.forEach(t => {
         const rad = t.angle * Math.PI / 180;
         const dirX = Math.cos(rad); const dirY = Math.sin(rad);
         const normX = -Math.sin(rad); const normY = Math.cos(rad);
         const rRoot = CORE_RADIUS;
         const rTip = CORE_RADIUS + Math.max(0, t.animLength); 
         const f = state.fillet;
         const hw = Math.min(t.animBaseW / 2, rRoot * 0.95); 
         const theta = Math.asin(hw / rRoot);
         const B1 = { x: rRoot * Math.cos(rad + theta), y: rRoot * Math.sin(rad + theta) };
         const B2 = { x: rRoot * Math.cos(rad - theta), y: rRoot * Math.sin(rad - theta) };
         const Tip1 = { x: dirX * rTip + normX * (t.animTipW / 2), y: dirY * rTip + normY * (t.animTipW / 2) };
         const Tip2 = { x: dirX * rTip - normX * (t.animTipW / 2), y: dirY * rTip - normY * (t.animTipW / 2) };
         const U1 = { x: (Tip1.x - B1.x) / (Math.hypot(Tip1.x - B1.x, Tip1.y - B1.y) || 1), y: (Tip1.y - B1.y) / (Math.hypot(Tip1.x - B1.x, Tip1.y - B1.y) || 1) };
         const U2 = { x: (Tip2.x - B2.x) / (Math.hypot(Tip2.x - B2.x, Tip2.y - B2.y) || 1), y: (Tip2.y - B2.y) / (Math.hypot(Tip2.x - B2.x, Tip2.y - B2.y) || 1) };
         
         ctx.beginPath();
         ctx.moveTo(rRoot * Math.cos(rad + theta + f/rRoot), rRoot * Math.sin(rad + theta + f/rRoot));
         ctx.quadraticCurveTo(B1.x, B1.y, B1.x + U1.x * f, B1.y + U1.y * f);
         ctx.lineTo(Tip1.x, Tip1.y);
         const segments = 3 + (t.id % 3);
         for (let i = 1; i < segments; i++) {
             const ang = (rad + Math.PI/2) + (-Math.PI) * (i / segments);
             ctx.lineTo(dirX * rTip + Math.cos(ang) * (t.animTipW/2), dirY * rTip + Math.sin(ang) * (t.animTipW/2));
         }
         ctx.lineTo(Tip2.x, Tip2.y);
         ctx.lineTo(B2.x + U2.x * f, B2.y + U2.y * f);
         ctx.quadraticCurveTo(B2.x, B2.y, rRoot * Math.cos(rad - theta - f/rRoot), rRoot * Math.sin(rad - theta - f/rRoot));
         ctx.closePath();
         ctx.fill();
      });

      ctx.beginPath(); ctx.arc(0, 0, CORE_RADIUS, 0, Math.PI * 2); ctx.fill();
      ctx.restore();
    };

    handleResize();
    // 首帧后再测一次，避免字体/布局延迟导致的初始尺寸抖动
    const resizeAfterPaint = requestAnimationFrame(handleResize);

    const resizeObserver = typeof ResizeObserver !== 'undefined'
      ? new ResizeObserver(() => handleResize())
      : null;
    resizeObserver?.observe(container);

    window.addEventListener('resize', handleResize);
    // 监听 window 上的鼠标移动，以支持更大范围的交互
    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseleave', handleMouseLeave);
    
    requestRef.current = requestAnimationFrame(animate);

    return () => {
      window.removeEventListener('resize', handleResize);
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('mouseleave', handleMouseLeave);
      resizeObserver?.disconnect();
      cancelAnimationFrame(resizeAfterPaint);
      if (requestRef.current) cancelAnimationFrame(requestRef.current);
    };
  }, []); // 依赖为空，使用 refs 管理可变状态

  return (
    <div 
      ref={containerRef} 
      className={className} 
      style={{ 
        width: '100%', 
        height: '100%', 
        ...style 
      }} 
      onClick={onClick}
    >
      <canvas 
        ref={canvasRef} 
        className="block touch-none"
        style={{ width: '100%', height: '100%', display: 'block' }} 
      />
    </div>
  );
};

export default ClaudeLogo;
