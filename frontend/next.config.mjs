/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  eslint: {
    ignoreDuringBuilds: true,
  },
  async rewrites() {
    // On Vercel without an external backend URL, let Next.js Route Handlers (/api/v1/[...slug]) handle requests natively
    if (process.env.VERCEL && !process.env.NEXT_PUBLIC_API_URL) {
      return [];
    }
    const apiBase =
      process.env.NEXT_PUBLIC_API_URL ||
      `http://localhost:${process.env.NEXT_PUBLIC_API_PORT || '8210'}`;
    return {
      fallback: [
        {
          source: '/api/v1/:path*',
          destination: `${apiBase}/api/v1/:path*`,
        },
      ],
    };
  },
};

export default nextConfig;
