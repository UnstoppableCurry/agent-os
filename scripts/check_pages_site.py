#!/usr/bin/env python3
"""Validate the docs/ GitHub Pages site: links, SEO, a11y basics, prefix rendering."""

from __future__ import annotations

import http.server
import re
import sys
import threading
import urllib.error
import urllib.request
from html.parser import HTMLParser
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOCS = ROOT / "docs"
PREFIX = "/agent-os"
CANONICAL_HOST = "https://unstoppablecurry.github.io/agent-os"
REQUIRED_PAGES = [
    "index.html",
    "architecture.html",
    "structure.html",
    "limitations.html",
    "404.html",
    "css/site.css",
    "robots.txt",
    "sitemap.xml",
    ".nojekyll",
]


class PageParser(HTMLParser):
    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.tags: list[str] = []
        self.hrefs: list[str] = []
        self.srcs: list[str] = []
        self.lang = ""
        self.title = ""
        self.metas: list[dict[str, str]] = []
        self.canonical = ""
        self.has_skip = False
        self.has_main = False
        self.has_h1 = False
        self.nav_current = 0
        self._in_title = False
        self._capture_title = []

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        self.tags.append(tag)
        ad = {k: (v or "") for k, v in attrs}
        if tag == "html":
            self.lang = ad.get("lang", "")
        if tag == "a":
            href = ad.get("href", "")
            if href:
                self.hrefs.append(href)
            if ad.get("class") == "skip-link" or "skip-link" in ad.get("class", ""):
                self.has_skip = True
            if ad.get("aria-current") == "page":
                self.nav_current += 1
        if tag == "link" and ad.get("rel") == "canonical":
            self.canonical = ad.get("href", "")
        if tag == "link" and ad.get("rel") == "stylesheet":
            self.hrefs.append(ad.get("href", ""))
        if tag in {"img", "script"}:
            src = ad.get("src", "")
            if src:
                self.srcs.append(src)
        if tag == "meta":
            self.metas.append(ad)
        if tag == "main" or ad.get("id") == "main":
            self.has_main = True
        if tag == "h1":
            self.has_h1 = True
        if tag == "title":
            self._in_title = True
            self._capture_title = []

    def handle_endtag(self, tag: str) -> None:
        if tag == "title" and self._in_title:
            self._in_title = False
            self.title = "".join(self._capture_title).strip()

    def handle_data(self, data: str) -> None:
        if self._in_title:
            self._capture_title.append(data)


def fail(errors: list[str], msg: str) -> None:
    errors.append(msg)


def resolve_internal(href: str, page: str) -> Path | None:
    if href.startswith("#"):
        return DOCS / page
    if href.startswith("mailto:") or href.startswith("https://") or href.startswith("http://"):
        return None
    if href.startswith(PREFIX + "/"):
        rel = href[len(PREFIX) + 1 :]
    elif href.startswith("/"):
        return Path("__outside__")  # flagged later
    else:
        rel = href
    rel = rel.split("#", 1)[0]
    if not rel or rel.endswith("/"):
        rel = (rel or "") + "index.html"
    return (DOCS / rel).resolve()


def check_html_file(rel: str, errors: list[str]) -> PageParser:
    text = (DOCS / rel).read_text(encoding="utf-8")
    parser = PageParser()
    parser.feed(text)
    parser.close()

    if parser.lang not in {"zh-Hans", "zh-CN", "zh"}:
        fail(errors, f"{rel}: html lang should be Chinese primary, got {parser.lang!r}")
    if not parser.title:
        fail(errors, f"{rel}: missing <title>")
    if not parser.has_h1:
        fail(errors, f"{rel}: missing <h1>")
    if not parser.has_main:
        fail(errors, f"{rel}: missing <main> or id=main")
    if not parser.has_skip and rel != "404.html":
        fail(errors, f"{rel}: missing skip link")

    desc = next((m.get("content", "") for m in parser.metas if m.get("name") == "description"), "")
    if rel != "404.html" and len(desc) < 40:
        fail(errors, f"{rel}: meta description missing or too short")

    if rel != "404.html":
        if not parser.canonical.startswith(CANONICAL_HOST):
            fail(errors, f"{rel}: canonical should start with {CANONICAL_HOST}, got {parser.canonical!r}")
        if parser.nav_current < 1:
            fail(errors, f"{rel}: current nav item (aria-current=page) missing")

    forbidden = ["lorem ipsum", "100万用户", "app store 精选", "截图如下"]
    low = text.lower()
    for phrase in forbidden:
        if phrase in low:
            fail(errors, f"{rel}: unexpected invented phrase {phrase!r}")

    # No <img> product shots
    if parser.srcs:
        fail(errors, f"{rel}: unexpected img/script src {parser.srcs}")

    for href in parser.hrefs:
        if not href:
            fail(errors, f"{rel}: empty href")
            continue
        if href.startswith("https://github.com/unstoppablecurry/agent-os"):
            continue
        if href.startswith("https://unstoppablecurry.github.io/agent-os"):
            continue
        if href.startswith("http://") or href.startswith("https://"):
            fail(errors, f"{rel}: unexpected external link {href}")
            continue
        target = resolve_internal(href, rel)
        if target is None:
            continue
        if str(target) == "__outside__" or not str(target).startswith(str(DOCS)):
            fail(errors, f"{rel}: href escapes docs/ : {href}")
            continue
        if not target.exists():
            fail(errors, f"{rel}: broken link {href} -> {target}")

    return parser


class PrefixHandler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=str(DOCS), **kwargs)

    def translate_path(self, path: str) -> str:
        if path.startswith(PREFIX + "/"):
            path = path[len(PREFIX) :] or "/"
        elif path == PREFIX:
            path = "/"
        return super().translate_path(path)

    def log_message(self, fmt: str, *args) -> None:  # noqa: A003
        return


def check_http_render(errors: list[str]) -> None:
    server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), PrefixHandler)
    port = server.server_address[1]
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    base = f"http://127.0.0.1:{port}{PREFIX}"
    try:
        for path in ["/", "/architecture.html", "/structure.html", "/limitations.html", "/css/site.css"]:
            url = base + path
            try:
                with urllib.request.urlopen(url, timeout=3) as resp:
                    body = resp.read()
                    ctype = resp.headers.get_content_type()
                    if resp.status != 200:
                        fail(errors, f"HTTP {resp.status} for {url}")
                    if path.endswith(".css"):
                        if "css" not in ctype and not body.startswith(b":root") and b"--bg" not in body:
                            fail(errors, f"{url}: CSS did not render as expected")
                        if b"--bg" not in body:
                            fail(errors, f"{url}: stylesheet missing expected tokens")
                    else:
                        if b"<html" not in body.lower() and b"<!doctype html>" not in body.lower():
                            fail(errors, f"{url}: HTML missing doctype/html")
                        if path == "/" and "AgentOS".encode() not in body:
                            fail(errors, f"{url}: home page missing AgentOS")
            except urllib.error.URLError as exc:
                fail(errors, f"fetch {url}: {exc}")
    finally:
        server.shutdown()


def check_sitemap_and_robots(errors: list[str]) -> None:
    sitemap = (DOCS / "sitemap.xml").read_text(encoding="utf-8")
    for slug in ["/", "/architecture.html", "/structure.html", "/limitations.html"]:
        loc = CANONICAL_HOST.rstrip("/") + ("" if slug == "/" else slug)
        # homepage loc has trailing slash in our sitemap
        if slug == "/":
            loc = CANONICAL_HOST + "/"
        if loc not in sitemap:
            fail(errors, f"sitemap.xml missing {loc}")
    robots = (DOCS / "robots.txt").read_text(encoding="utf-8")
    if "Sitemap:" not in robots:
        fail(errors, "robots.txt missing Sitemap")


def main() -> int:
    errors: list[str] = []
    for rel in REQUIRED_PAGES:
        if not (DOCS / rel).exists():
            fail(errors, f"missing required file docs/{rel}")

    html_files = sorted(p.name for p in DOCS.glob("*.html"))
    for name in html_files:
        check_html_file(name, errors)

    check_sitemap_and_robots(errors)
    check_http_render(errors)

    # CSS: no named-color-only reliance; check focus + reduced motion
    css = (DOCS / "css" / "site.css").read_text(encoding="utf-8")
    for token in [":focus-visible", "prefers-reduced-motion", "prefers-color-scheme", "skip-link"]:
        if token not in css:
            fail(errors, f"css/site.css missing {token}")

    if errors:
        print("Pages site checks failed:", file=sys.stderr)
        for item in errors:
            print(f"  - {item}", file=sys.stderr)
        return 1
    print(f"OK: {len(html_files)} HTML pages, links, SEO/a11y basics, and /agent-os/ rendering.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
