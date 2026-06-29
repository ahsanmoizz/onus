'use client';

import Image from 'next/image';

type BrandLogoProps = {
  className?: string;
  imageClassName?: string;
};

export function BrandLogo({ className = '', imageClassName = 'h-11 w-auto' }: BrandLogoProps) {
  const basePath = process.env.NEXT_PUBLIC_SITE_BASE_PATH || '';

  return (
    <span className={`inline-flex items-center ${className}`}>
      <Image
        src={`${basePath}/onus1.png`}
        alt="Onus - Control, Verify, Protect"
        width={260}
        height={120}
        unoptimized
        className={`${imageClassName} rounded-sm object-contain`}
      />
    </span>
  );
}
