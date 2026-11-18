#!/usr/bin/env python3
"""Valida HTML de apps/landing: cierre de etiquetas y assets relativos ./"""

from __future__ import annotations

import pathlib
import sys
from html.parser import HTMLParser

VOID = {
    "area",
    "base",
    "br",
    "col",
    "embed",
    "hr",
    "img",
    "input",
    "link",
    "meta",
    "param",
    "source",
    "track",
    "wbr",
}

ROOT = pathlib.Path("apps/landing")


class Checker(HTMLParser):
    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.stack: list[str] = []
        self.assets: list[str] = []
        self.errors: list[str] = []

    def _collect_assets(self, attrs: list[tuple[str, str | None]]) -> None:
        data = dict(attrs)
        for key in ("href", "src"):
            val = data.get(key)
            if val and val.startswith("./"):
                self.assets.append(val)

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        self._collect_assets(attrs)
        if tag not in VOID:
            self.stack.append(tag)

    def handle_startendtag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        self._collect_assets(attrs)

    def handle_endtag(self, tag: str) -> None:
        if tag in VOID:
            return
        if not self.stack:
            self.errors.append(f"cierre inesperado </{tag}>")
            return
        if self.stack[-1] != tag:
            expected = self.stack[-1]
            self.errors.append(f"cierre cruzado: esperado </{expected}>, llegó </{tag}>")
            return
        self.stack.pop()


def main() -> int:
    html_path = ROOT / "index.html"
    if not html_path.is_file():
        print(f"❌ falta {html_path}", file=sys.stderr)
        return 1

    text = html_path.read_text(encoding="utf-8")
    checker = Checker()
    try:
        checker.feed(text)
        checker.close()
    except Exception as exc:  # noqa: BLE001 — cualquier fallo de parseo bloquea
        print(f"❌ landing HTML: {exc}", file=sys.stderr)
        return 1

    if checker.stack:
        print(
            "❌ landing HTML: etiquetas sin cerrar:",
            " ".join(checker.stack),
            file=sys.stderr,
        )
        return 1
    if checker.errors:
        print(f"❌ landing HTML: {checker.errors[0]}", file=sys.stderr)
        return 1
    if not checker.assets:
        print("❌ landing: no hay assets relativos ./", file=sys.stderr)
        return 1

    root_r = ROOT.resolve()
    for rel in checker.assets:
        path = (ROOT / rel[2:]).resolve()
        try:
            path.relative_to(root_r)
        except ValueError:
            print(f"❌ landing: asset fuera de apps/landing: {rel}", file=sys.stderr)
            return 1
        if not path.is_file():
            print(f"❌ landing: falta asset {rel}", file=sys.stderr)
            return 1

    print("✓ Landing HTML + assets OK")
    return 0


if __name__ == "__main__":
    sys.exit(main())
