/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  async rewrites() {
    const apiPort = process.env.NEXT_PUBLIC_API_PORT || '8210';
    return [
      {
        source: '/api/v1/:path*',
        destination: `http://localhost:${apiPort}/api/v1/:path*`,
      },
    ];
  },
};

export default nextConfig;
