import { NextRequest, NextResponse } from "next/server";
import { handleCloudApiRequest } from "@/lib/cloudStore";

async function handleRequest(req: NextRequest, { params }: { params: { slug: string[] } }) {
  const slugPath = "/" + (params.slug || []).join("/");
  const search = req.nextUrl.search || "";
  const endpoint = `${slugPath}${search}`;
  const method = req.method;

  let bodyStr: string | undefined = undefined;
  if (method !== "GET" && method !== "HEAD") {
    try {
      bodyStr = await req.text();
    } catch {
      bodyStr = "{}";
    }
  }

  const data = handleCloudApiRequest(endpoint, {
    method,
    body: bodyStr,
  });

  return NextResponse.json(data);
}

export async function GET(req: NextRequest, ctx: { params: { slug: string[] } }) {
  return handleRequest(req, ctx);
}

export async function POST(req: NextRequest, ctx: { params: { slug: string[] } }) {
  return handleRequest(req, ctx);
}

export async function PATCH(req: NextRequest, ctx: { params: { slug: string[] } }) {
  return handleRequest(req, ctx);
}

export async function PUT(req: NextRequest, ctx: { params: { slug: string[] } }) {
  return handleRequest(req, ctx);
}

export async function DELETE(req: NextRequest, ctx: { params: { slug: string[] } }) {
  return handleRequest(req, ctx);
}
