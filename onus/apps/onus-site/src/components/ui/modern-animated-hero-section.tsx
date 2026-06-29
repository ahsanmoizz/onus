'use client';

import { useEffect, useRef, useState } from 'react';

type Character = {
  char: string;
  x: number;
  y: number;
  speed: number;
};

class TextScramble {
  private el: HTMLElement;
  private chars = '!<>-_\\/[]{}=+*^?#';
  private queue: Array<{ from: string; to: string; start: number; end: number; char?: string }> = [];
  private frame = 0;
  private frameRequest = 0;
  private resolve: () => void = () => {};

  constructor(el: HTMLElement) {
    this.el = el;
    this.update = this.update.bind(this);
  }

  setText(newText: string) {
    const oldText = this.el.innerText;
    const length = Math.max(oldText.length, newText.length);
    const promise = new Promise<void>((resolve) => {
      this.resolve = resolve;
    });

    this.queue = [];
    for (let i = 0; i < length; i++) {
      const from = oldText[i] || '';
      const to = newText[i] || '';
      const start = Math.floor(Math.random() * 18);
      const end = start + Math.floor(Math.random() * 22);
      this.queue.push({ from, to, start, end });
    }

    cancelAnimationFrame(this.frameRequest);
    this.frame = 0;
    this.update();
    return promise;
  }

  update() {
    let output = '';
    let complete = 0;

    for (let i = 0; i < this.queue.length; i++) {
      const item = this.queue[i];
      if (this.frame >= item.end) {
        complete++;
        output += item.to;
      } else if (this.frame >= item.start) {
        if (!item.char || Math.random() < 0.28) {
          item.char = this.chars[Math.floor(Math.random() * this.chars.length)];
        }
        output += `<span class="dud">${item.char}</span>`;
      } else {
        output += item.from;
      }
    }

    this.el.innerHTML = output;
    if (complete === this.queue.length) {
      this.resolve();
    } else {
      this.frameRequest = requestAnimationFrame(this.update);
      this.frame++;
    }
  }

  destroy() {
    cancelAnimationFrame(this.frameRequest);
  }
}

function ScrambledPhrase() {
  const elementRef = useRef<HTMLSpanElement>(null);
  const scramblerRef = useRef<TextScramble | null>(null);

  useEffect(() => {
    if (!elementRef.current) return;

    const phrases = [
      'agents',
      'actions',
      'prompts',
      'approvals',
      'evidence',
      'rollbacks',
      'sessions',
    ];
    let active = true;
    let counter = 0;
    const scrambler = new TextScramble(elementRef.current);
    scramblerRef.current = scrambler;

    const next = () => {
      if (!active) return;
      scrambler.setText(phrases[counter]).then(() => {
        counter = (counter + 1) % phrases.length;
        window.setTimeout(next, 1500);
      });
    };

    next();

    return () => {
      active = false;
      scrambler.destroy();
    };
  }, []);

  return (
    <span
      ref={elementRef}
      className="font-mono text-[#ffe55c] drop-shadow-[0_0_20px_rgba(255,229,92,0.26)]"
    >
      agents
    </span>
  );
}

function createCharacters(): Character[] {
  const allChars = 'ONUSCONTROLVERIFYPROTECT0123456789[]{}<>/\\#@';
  return Array.from({ length: 150 }, () => ({
    char: allChars[Math.floor(Math.random() * allChars.length)],
    x: Math.random() * 100,
    y: Math.random() * 100,
    speed: 0.04 + Math.random() * 0.12,
  }));
}

export function RainingOnusHero() {
  const [characters, setCharacters] = useState<Character[]>([]);
  const [activeIndices, setActiveIndices] = useState<Set<number>>(new Set());

  useEffect(() => {
    setCharacters(createCharacters());
  }, []);

  useEffect(() => {
    const flickerInterval = window.setInterval(() => {
      setActiveIndices((previous) => {
        const next = new Set<number>();
        const count = Math.min(7, Math.max(3, previous.size + 1));
        for (let i = 0; i < count; i++) {
          next.add(Math.floor(Math.random() * Math.max(characters.length, 1)));
        }
        return next;
      });
    }, 90);

    return () => window.clearInterval(flickerInterval);
  }, [characters.length]);

  useEffect(() => {
    let animationFrameId = 0;
    let last = 0;

    const updatePositions = (timestamp: number) => {
      if (timestamp - last > 42) {
        setCharacters((previous) =>
          previous.map((char) => {
            const y = char.y + char.speed;
            if (y < 105) return { ...char, y };
            const chars = 'ONUSCONTROLVERIFYPROTECT0123456789[]{}<>/\\#@';
            return {
              ...char,
              x: Math.random() * 100,
              y: -5,
              char: chars[Math.floor(Math.random() * chars.length)],
            };
          }),
        );
        last = timestamp;
      }
      animationFrameId = requestAnimationFrame(updatePositions);
    };

    animationFrameId = requestAnimationFrame(updatePositions);
    return () => cancelAnimationFrame(animationFrameId);
  }, []);

  return (
    <div className="pointer-events-none absolute inset-0 overflow-hidden">
      {characters.map((char, index) => (
        <span
          key={index}
          className={`absolute select-none font-mono transition-[color,opacity,transform,text-shadow] duration-100 ${
            activeIndices.has(index)
              ? 'z-10 text-[#ffe55c] opacity-100'
              : 'text-white/18 opacity-40'
          }`}
          style={{
            left: `${char.x}%`,
            top: `${char.y}%`,
            transform: `translate(-50%, -50%) ${activeIndices.has(index) ? 'scale(1.18)' : 'scale(1)'}`,
            textShadow: activeIndices.has(index) ? '0 0 14px rgba(255,229,92,0.62)' : 'none',
            fontSize: activeIndices.has(index) ? '1.22rem' : '1rem',
            willChange: 'transform, top',
          }}
        >
          {char.char}
        </span>
      ))}
      <style jsx global>{`
        .dud {
          color: #ffe55c;
          opacity: 0.78;
        }
      `}</style>
    </div>
  );
}

export function OnusScrambleLine() {
  return (
    <span>
      Govern <ScrambledPhrase /> before they touch production.
    </span>
  );
}
