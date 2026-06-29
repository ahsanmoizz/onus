/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  transpilePackages: ['@onus/ui', '@onus/api-client', '@onus/types'],
  async rewrites() {
    return [
      {
        source: '/api/:path*',
        destination: 'http://127.0.0.1:9090/api/:path*',
      },
    ];
  },
};
module.exports = nextConfig;
