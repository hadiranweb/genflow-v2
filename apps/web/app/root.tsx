import {
  Links,
  Meta,
  Outlet,
  Scripts,
  ScrollRestoration,
  useLoaderData,
} from "@remix-run/react";
import type { LinksFunction, LoaderFunctionArgs } from "@remix-run/node";
import styles from "./tailwind.css?url";
import { GenFlowApi } from "./lib/api";

export const links: LinksFunction = () => [
  { rel: "stylesheet", href: styles },
  { rel: "icon", href: "/favicon.svg", type: "image/svg+xml" },
];

export async function loader({ request }: LoaderFunctionArgs) {
  const api = new GenFlowApi();
  const health = await api.health().catch(() => null);
  return { lang: "fa", dir: "rtl" as const, health };
}

export default function App() {
  const { lang, dir } = useLoaderData<typeof loader>();

  return (
    <html lang={lang} dir={dir}>
      <head>
        <meta charSet="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <Meta />
        <Links />
      </head>
      <body className="min-h-screen bg-gray-50">
        <Outlet />
        <ScrollRestoration />
        <Scripts />
      </body>
    </html>
  );
}
