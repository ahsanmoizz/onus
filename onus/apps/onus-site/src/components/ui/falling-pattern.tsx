'use client';

import { motion } from 'framer-motion';
import { cn } from '@/lib/utils';

interface FallingPatternProps {
  className?: string;
  color?: string;
  backgroundColor?: string;
  duration?: number;
  blurIntensity?: number;
  density?: number;
}

export function FallingPattern({
  className,
  color = 'rgba(255,255,255,0.34)',
  backgroundColor = '#000000',
  duration = 32,
  blurIntensity = 1,
  density = 1,
}: FallingPatternProps) {
  const generateBackgroundImage = () => {
    const patterns = [
      `radial-gradient(4px 100px at 0px 235px, ${color}, transparent)`,
      `radial-gradient(4px 100px at 300px 235px, ${color}, transparent)`,
      `radial-gradient(1.5px 1.5px at 150px 117.5px, ${color} 100%, transparent 150%)`,
      `radial-gradient(4px 100px at 0px 252px, ${color}, transparent)`,
      `radial-gradient(4px 100px at 300px 252px, ${color}, transparent)`,
      `radial-gradient(1.5px 1.5px at 150px 126px, ${color} 100%, transparent 150%)`,
      `radial-gradient(4px 100px at 0px 150px, ${color}, transparent)`,
      `radial-gradient(4px 100px at 300px 150px, ${color}, transparent)`,
      `radial-gradient(1.5px 1.5px at 150px 75px, ${color} 100%, transparent 150%)`,
      `radial-gradient(4px 100px at 0px 253px, ${color}, transparent)`,
      `radial-gradient(4px 100px at 300px 253px, ${color}, transparent)`,
      `radial-gradient(1.5px 1.5px at 150px 126.5px, ${color} 100%, transparent 150%)`,
    ];
    return patterns.join(', ');
  };

  const backgroundSizes = [
    '300px 235px',
    '300px 235px',
    '300px 235px',
    '300px 252px',
    '300px 252px',
    '300px 252px',
    '300px 150px',
    '300px 150px',
    '300px 150px',
    '300px 253px',
    '300px 253px',
    '300px 253px',
  ].join(', ');

  const startPositions =
    '0px 220px, 3px 220px, 151.5px 337.5px, 25px 24px, 28px 24px, 176.5px 150px, 50px 16px, 53px 16px, 201.5px 91px, 75px 224px, 78px 224px, 226.5px 230.5px';
  const endPositions =
    '0px 6800px, 3px 6800px, 151.5px 6917.5px, 25px 13632px, 28px 13632px, 176.5px 13758px, 50px 5416px, 53px 5416px, 201.5px 5491px, 75px 17175px, 78px 17175px, 226.5px 17301.5px';

  return (
    <div className={cn('relative h-full w-full overflow-hidden', className)}>
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ duration: 0.25 }}
        className="size-full"
      >
        <motion.div
          className="relative z-0 size-full"
          style={{
            backgroundColor,
            backgroundImage: generateBackgroundImage(),
            backgroundSize: backgroundSizes,
          }}
          variants={{
            initial: { backgroundPosition: startPositions },
            animate: {
              backgroundPosition: [startPositions, endPositions],
              transition: { duration, ease: 'linear', repeat: Number.POSITIVE_INFINITY },
            },
          }}
          initial="initial"
          animate="animate"
        />
      </motion.div>
      <div
        className="absolute inset-0 z-10"
        style={{
          backdropFilter: `blur(${blurIntensity}px)`,
          backgroundImage: `radial-gradient(circle at 50% 50%, transparent 0, transparent 2px, ${backgroundColor} 2px)`,
          backgroundSize: `${8 * density}px ${8 * density}px`,
        }}
      />
    </div>
  );
}
