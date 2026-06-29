/** @type {import('next').NextConfig} */
const basePath = process.env.NEXT_PUBLIC_SITE_BASE_PATH || '';

const nextConfig = {
  reactStrictMode: true,
  transpilePackages: ['@onus/ui'],
  output: 'export',
  basePath,
  assetPrefix: basePath || undefined,
  eslint: {
    ignoreDuringBuilds: false,
  },
};
module.exports = nextConfig;
