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
        else:
            entry = page.locator('.appmap [data-goto="exception"]')
            label, scope = page.evaluate("""l => {
                const d = window.ODINA_STRINGS[l]; return [d['exception.title'], d['tip.exception']];
            }""", lang)
            assert "—" not in label and (lang != "ar" or label == "الاستثناءات"), (lang, label)
            expect(entry).to_have_accessible_name(label)
            expect(entry).to_have_accessible_description(scope)
            assert page.locator('.list-tip [role="tooltip"]').evaluate_all("""(tips, l) =>
                tips.every(t => t.textContent && t.textContent === window.ODINA_STRINGS[l][t.dataset.k])
            """, lang), lang


def check_list_help(page: Page) -> None:
    entry = page.locator('.appmap [data-goto="exception"]')
    row = entry.locator("..")
    tip = row.locator('[role="tooltip"]')
    info = row.locator("summary")
    initial_url = page.url
    expect(tip).to_be_hidden()
    entry.hover()
    expect(tip).to_be_visible()
    tip.hover()  # The explanation must remain readable when the pointer moves onto it.
    expect(tip).to_be_visible()
    page.keyboard.press("Escape")
    expect(tip).to_be_hidden()

    entry.focus()
    page.keyboard.press("Tab")
    expect(info).to_be_focused()
    expect(tip).to_be_visible()
    page.keyboard.press("Escape")
    expect(tip).to_be_hidden()
    expect(info).to_be_focused()
    info.press("Enter")
    expect(tip).to_be_visible()
    expect(page).to_have_url(initial_url)
    page.keyboard.press("Shift+Tab")
    expect(entry).to_be_focused()
    entry.press("Enter")  # The name navigates even while its description is open.
    screen = page.locator("#screen-exception")
    expect(screen).to_be_focused()
    expect(screen.locator("h2")).to_have_text("الاستثناءات")
    expect(tip).to_be_hidden()
    expect(screen.locator(".excbadge")).to_contain_text("14:30")
    for card in screen.locator(".exccard").all():
        expect(card.locator("[role=tooltip]")).to_be_hidden()
        expect(card.locator(".st")).to_be_visible()
        expect(card.locator(".conseq")).to_be_visible()
    expect(screen.locator(".exccard.live .st")).to_contain_text("14:30")
    expect(screen.locator('.exccard.live [data-k="exception.action.revoke"]')).to_be_visible()
    page.keyboard.press("Escape")
    expect(entry).to_be_focused()
    expect(page).to_have_url(initial_url)


def check_touch_help(page: Page, base: str) -> None:
    page.goto(base + "index.html")
    entry = page.locator('#screen-proactive [data-goto="exception"]')
    row = entry.locator("..")
    info, tip = row.locator("summary"), row.locator("[role=tooltip]")
    # A tooltip above this entry would be outside the viewport.
    entry.evaluate("el => window.scrollBy(0, el.getBoundingClientRect().top - 8)")
    info.tap()
    expect(tip).to_be_visible()
    expect(page).to_have_url(base + "index.html")
    rect = tip.bounding_box()
    view = page.viewport_size
    assert rect and view and 0 <= rect["x"] <= view["width"] - rect["width"], rect
    assert 0 <= rect["y"] <= view["height"] - rect["height"], rect
    info.tap()
    expect(tip).to_be_hidden()
    entry.tap()
    screen = page.locator("#screen-exception")
    expect(screen).to_be_focused()
    card = screen.locator(".exccard.live")
    card.locator("summary").tap()
    expect(card.get_by_role("tooltip")).to_be_visible()
    assert card.evaluate("""c => c.querySelector('[role=tooltip]').getBoundingClientRect().bottom
        <= c.querySelector('.st').getBoundingClientRect().top"""), "help covers the rule state"
    card.locator(".conseq").tap()  # Outside tap dismisses, without changing the rule.
    expect(card.locator("[role=tooltip]")).to_be_hidden()
    expect(card.locator(".st")).to_contain_text("14:30")
    screen.locator("[data-back]").tap()
    expect(entry).to_be_focused()
    expect(page).to_have_url(base + "index.html")
    assert len(page.context.pages) == 1, "local help opened another tab"


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
    check_list_help(page)
    page.locator('.appbar [data-goto="settings"]').click()
    appearance = page.locator('#screen-settings summary[data-k="settings.appearance"]')
    hint = appearance.locator("..").locator('[role="tooltip"]')
    expect(hint).to_be_hidden()
    appearance.hover()
    expect(hint).to_be_visible()
    page.keyboard.press("Escape")
    expect(hint).to_be_hidden()
    menu = page.locator("[data-menu]")
    menu.click()
    numbers = page.locator('.menu [data-goto="numbers"]')
    info = numbers.locator("..").locator("summary")
    info.click()
    expect(numbers.locator("..").get_by_role("tooltip")).to_be_visible()
    page.keyboard.press("Escape")  # Dismiss help first; do not leave the menu or the screen.
    expect(menu).to_have_attribute("aria-expanded", "true")
    expect(info).to_be_focused()
    expect(numbers.locator("..").locator("[role=tooltip]")).to_be_hidden()
    numbers.click()
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
                base = f"http://127.0.0.1:{server.server_port}/"
                viewport = {"width": 360, "height": 800}
                page = browser.new_page(viewport=viewport, reduced_motion="reduce")
                review(page, base)
                assert len(page.context.pages) == 1, "local help opened another tab"
                touch = browser.new_page(viewport=viewport, reduced_motion="reduce", has_touch=True, is_mobile=True)
                check_touch_help(touch, base)
                browser.close()
        finally:
            server.shutdown()
    print("site smoke: 10 locales on both pages · list help (hover, keyboard, touch) · alert → quarantine → back")


if __name__ == "__main__":
    main()
