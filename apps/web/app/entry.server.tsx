import type { EntryContext } from "@remix-run/node";
import { RemixServer } from "@remix-run/react";
import { renderToString } from "react-dom/server";

export default function handleRequest(
  request: Request,
  responseStatusCode: number,
  responseHeaders: Headers,
  remixContext: EntryContext
) {
  let html = renderToString(
    <RemixServer context={remixContext} url={request.url} />
  );

  if (responseStatusCode >= 500 && responseStatusCode < 600) {
    responseHeaders.set("Retry-After", "5");
  }

  return new Response("<!DOCTYPE html>" + html, {
    headers: { "Content-Type": "text/html", ...Object.fromEntries(responseHeaders) },
    status: responseStatusCode,
  });
}
