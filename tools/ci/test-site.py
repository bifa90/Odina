#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
# Copyright (c) 2026 bifa90 — Odina developer tools (see LICENSE-TOOLS)
"""Smoke-test the actual staged pages, not a DOM assembled from the sources.

Run tools/deploy-site.py --dry-run first. Needs Playwright and Chromium
(CHROME_BIN, system Chrome, or `python -m playwright install chromium`).
Only the staging directory is served, on a temporary loopback port.
"""

import os
import shutil
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from threading import Thread

from playwright.sync_api import Page, expect, sync_playwright

STAGE = Path(__file__).resolve().parents[2] / "site" / "_build"
DRAFT = {"de", "fr", "pt-BR"}


def check_picker(page: Page, picker: str, sample: str, langs: list[str]) -> None:
    select = page.locator(picker)
    assert select.locator("option").evaluate_all("options => options.map(o => o.value)") == langs
    for lang in langs:
        select.select_option(lang)
        expect(page.locator("html")).to_have_attribute("lang", lang)
        expect(page.locator("html")).to_have_attribute("dir", "rtl" if lang == "ar" else "ltr")
        expected = page.evaluate("([l, k]) => window.ODINA_STRINGS[l][k]", [lang, sample])
        expect(page.locator(f'[data-k="{sample}"]').first).to_have_text(expected)
        assert page.locator("#draftflag").is_visible() == (lang in DRAFT), lang
        unfilled = page.locator("[data-k]").evaluate_all(
            r"els => els.filter(el => /\{\s*\$/.test(el.textContent)).map(el => el.dataset.k)"
        )
        assert not unfilled, (lang, unfilled)
        assert page.evaluate("document.documentElement.scrollWidth <= innerWidth"), lang
        if picker == "#langsel":
            assert not page.evaluate("window.ODINA_MISSING"), lang


def review(page: Page, base: str) -> None:
    errors = []
    page.on("pageerror", lambda exc: errors.append(str(exc)))

    def check_response(response):
        if response.url.startswith(base) and response.status >= 400:
            errors.append(f"HTTP {response.status}: {response.url}")

    page.on("response", check_response)
    page.goto(base + "index.html")
    assert not errors, errors  # A ReferenceError on boot otherwise looks like an inert picker.
    langs = page.evaluate("window.ODINA_ORDER")
    assert langs and len(langs) == len(set(langs)) == 10, langs
    check_picker(page, "#lang", "state.protected", langs)
    page.locator('.langcard[value="ar"]').click()
    expect(page.locator("#lang")).to_have_value("ar")
    expect(page.locator('[data-k="alert.confirmed.body"]')).to_contain_text("PhotoTool.exe")
    page.locator('.appbar [data-goto="settings"]').click()
    menu = page.locator("[data-menu]")
    menu.click()
    page.locator('.menu [data-goto="numbers"]').click()
    expect(page.locator("#screen-numbers")).to_be_focused()
    page.locator('[data-num="hindi"]').click()
    expect(page.locator('[data-k="shield.sub"]')).to_contain_text("٣")
    expect(page.locator('[data-k="exception.badge"]')).to_contain_text("14:30")
    expect(page.locator('[data-k="usb.scanning"]')).to_contain_text("USB_32GB")
    page.keyboard.press("Escape")
    expect(menu).to_be_focused()
    page.keyboard.press("Escape")
    expect(page.locator('.appbar [data-goto="settings"]')).to_be_focused()

    alerts = page.locator('.appbar [data-goto="alerts"]')
    alerts.click()
    cta = page.locator('[data-k="alert.confirmed.cta"]')
    cta.click()
    quarantine = page.locator("#screen-quarantine")
    expect(quarantine).to_be_focused()
    expect(quarantine.locator(".qname").first).to_have_text("PhotoTool.exe")
    quarantine.locator("[data-back]").click()
    expect(cta).to_be_focused()
    page.keyboard.press("Escape")
    expect(alerts).to_be_focused()

    page.select_option("#lang", "de")
    page.locator('a[href="signing.html"]').click()
    expect(page.locator("#langsel")).to_have_value("de")
    check_picker(page, "#langsel", "signing.title", langs)
    page.select_option("#langsel", "en")
    page.locator('a[href="index.html"]').click()
    expect(page.locator("#lang")).to_have_value("en")
    assert not errors, errors


def main() -> None:
    if not (STAGE / "index.html").is_file():
        raise SystemExit("Build the site first: python3 tools/deploy-site.py --dry-run")
    handler = partial(SimpleHTTPRequestHandler, directory=str(STAGE))
    with ThreadingHTTPServer(("127.0.0.1", 0), handler) as server:
        Thread(target=server.serve_forever, daemon=True).start()
        try:
            with sync_playwright() as pw:
                chrome = os.environ.get("CHROME_BIN") or shutil.which("google-chrome")
                browser = pw.chromium.launch(executable_path=chrome)
                page = browser.new_page(viewport={"width": 360, "height": 800}, reduced_motion="reduce")
                review(page, f"http://127.0.0.1:{server.server_port}/")
                browser.close()
        finally:
            server.shutdown()
    print("site smoke: staged preview and signing page, 10 locales, alert → quarantine → back")


if __name__ == "__main__":
    main()
