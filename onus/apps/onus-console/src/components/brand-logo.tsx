'use client';

import Image from 'next/image';

type BrandLogoProps = {
  className?: string;
  imageClassName?: string;
};

export function BrandLogo({ className = '', imageClassName = 'h-9 w-auto' }: BrandLogoProps) {
  return (
    <span className={`inline-flex items-center ${className}`}>
      <Image
        src="/onus1.png"
        alt="Onus - Control, Verify, Protect"
        width={260}
        height={120}
        unoptimized
        className={`${imageClassName} rounded-sm object-contain`}
      />
    </span>
  );
}
