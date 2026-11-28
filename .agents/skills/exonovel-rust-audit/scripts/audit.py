#!/usr/bin/env python3
"""Deterministic static structural audit for ExoNovel Rust crates."""

from __future__ import annotations

import glob
import os
import re
import sys


def strip_comments_and_strings(src: str) -> str:
    """Replace comments and string literal bodies with spaces while keeping line breaks."""
    out: list[str] = []
    i = 0
    n = len(src)
    while i < n:
        ch = src[i]
        nxt = src[i + 1] if i + 1 < n else ""
        if ch == "/" and nxt == "/":
            while i < n and src[i] != "\n":
                out.append(" ")
                i += 1
        elif ch == "/" and nxt == "*":
            out.extend("  ")
            i += 2
            depth = 1
            while i < n and depth > 0:
                if src[i] == "/" and i + 1 < n and src[i + 1] == "*":
                    depth += 1
                    out.extend("  ")
                    i += 2
                elif src[i] == "*" and i + 1 < n and src[i + 1] == "/":
                    depth -= 1
                    out.extend("  ")
                    i += 2
                else:
                    out.append("\n" if src[i] == "\n" else " ")
                    i += 1
        elif ch == "r" and nxt in ('"', "#"):
            hashes = 0
            j = i + 1
            while j < n and src[j] == "#":
                hashes += 1
                j += 1
            if j < n and src[j] == '"':
                out.extend(" " * (j - i + 1))
                i = j + 1
                end_seq = '"' + ("#" * hashes)
                while i < n:
                    if src.startswith(end_seq, i):
                        out.extend(" " * len(end_seq))
                        i += len(end_seq)
                        break
                    out.append("\n" if src[i] == "\n" else " ")
                    i += 1
                continue
            out.append(ch)
            i += 1
        elif ch == '"':
            out.append('"')
            i += 1
            while i < n:
                if src[i] == "\\":
                    out.extend("  ")
                    i += 2
                elif src[i] == '"':
                    out.append('"')
                    i += 1
                    break
                else:
                    out.append("\n" if src[i] == "\n" else " ")
                    i += 1
        else:
            out.append(ch)
            i += 1
    return "".join(out)


def find_matching_brace(text: str, open_idx: int, open_ch: str = "{", close_ch: str = "}") -> int:
    depth = 0
    for idx in range(open_idx, len(text)):
        if text[idx] == open_ch:
            depth += 1
        elif text[idx] == close_ch:
            depth -= 1
            if depth == 0:
                return idx
    return -1


def check_brace_balance(path: str, masked: str) -> list[str]:
    stack: list[tuple[str, int]] = []
    pairs = {"(": ")", "[": "]", "{": "}"}
    closing = {v: k for k, v in pairs.items()}
    line = 1
    for ch in masked:
        if ch == "\n":
            line += 1
        elif ch in pairs:
            stack.append((ch, line))
        elif ch in closing:
            if not stack or stack[-1][0] != closing[ch]:
                return [f"{path}:{line}: unmatched closing `{ch}`"]
            stack.pop()
    if stack:
        ch, ln = stack[-1]
        return [f"{path}:{ln}: unclosed `{ch}`"]
    return []


def check_iterator_move_captures(path: str, src: str, masked: str) -> list[str]:
    """Flag outer `let var = ...` (especially `.get()` / owned non-Copy values) moved into an inner `move` closure inside `.map(...)`."""
    errors: list[str] = []
    # Find enclosing closure blocks before `.into_iter().map(`
    for m in re.finditer(r"\.into_iter\(\)\.map\(\s*(?:move\s*)?\|([^|]*)\|\s*\{", masked):
        map_start = m.start()
        body_open = m.end() - 1
        body_end = find_matching_brace(masked, body_open, "{", "}")
        if body_end == -1:
            continue
        map_body = masked[body_open + 1 : body_end]
        # Collect outer `let ident = ...;` bindings declared in the preceding 40 lines at outer block scope
        prefix_start = max(0, masked.rfind("move || {", 0, map_start))
        outer_block = masked[prefix_start:map_start]
        outer_lets = set(re.findall(r"\blet\s+(?:mut\s+)?([a-z_][a-zA-Z0-9_]*)\s*=", outer_block))
        # Exclude variables shadowed inside the map body itself
        inner_lets = set(re.findall(r"\blet\s+(?:mut\s+)?([a-z_][a-zA-Z0-9_]*)\s*=", map_body))
        param_names = {p.strip().lstrip("mut ").split(":")[0].strip() for p in m.group(1).split(",") if p.strip()}
        candidate_outer = outer_lets - inner_lets - param_names

        # Check every inner `move ||` or `move |_|` closure inside map_body
        for inner in re.finditer(r"\bmove\s*\|[^|]*\|\s*", map_body):
            after = map_body[inner.end() :]
            if after.startswith("{"):
                end_rel = find_matching_brace(after, 0, "{", "}")
                closure_txt = after[: end_rel + 1] if end_rel != -1 else after[:300]
            else:
                stmt_end = after.find(",")
                nl_end = after.find("\n")
                cut = min([x for x in (stmt_end, nl_end, 220) if x != -1])
                closure_txt = after[:cut]

            # Look for non-method bare identifier usage of outer_let likely owning String/Vec (heuristic: outer assignment uses .get() without .len()/.is_empty() or to_string/clone/format)
            for ident in sorted(candidate_outer):
                if re.search(rf"\b{re.escape(ident)}\b(?!\s*\.\s*(?:get|get_untracked|set|update|with|tr|tr_untracked|anim_class)\s*\()", closure_txt):
                    assign_match = re.search(rf"\blet\s+(?:mut\s+)?{re.escape(ident)}\s*=\s*([^;]+);", outer_block)
                    rhs = assign_match.group(1) if assign_match else ""
                    # Likely Copy if literal bool/number or .len()/.is_empty()/.is_some()/.is_none()
                    if re.search(r"\b(true|false)\b|\.\s*(len|is_empty|is_some|is_none|chars)\s*\(", rhs):
                        continue
                    if re.search(r"\.\s*(get|to_string|clone)\s*\(|format!\s*\(", rhs):
                        ln = masked[: body_open + 1 + inner.start()].count("\n") + 1
                        errors.append(
                            f"{path}:{ln}: outer non-Copy `{ident}` moved into inner `move` closure inside `.into_iter().map(...)`"
                        )
    return errors


def check_no_target_dir() -> list[str]:
    if os.path.exists("target"):
        return ["target/ directory exists locally; remove it to comply with 4 GB container limits."]
    return []


def check_ftl_keys() -> list[str]:
    app_keys: set[str] = set()
    for p in glob.glob("crates/app/src/*.rs"):
        txt = open(p, encoding="utf-8").read()
        for m in re.finditer(r'i18n\.tr(?:_untracked)?\(\s*"([a-z0-9_]+)"', txt):
            app_keys.add(m.group(1).replace("_", "-"))
    i18n_src = open("crates/server/src/i18n.rs", encoding="utf-8").read()
    defined = set(re.findall(r"^\s*([a-z0-9][a-z0-9-]*)\s*=", i18n_src, flags=re.MULTILINE))
    defined |= {k.replace("_", "-") for k in re.findall(r"\b([a-z0-9_]+)\s*:\s*&'static str", i18n_src)}
    missing = sorted(k for k in app_keys if k not in defined)
    if missing:
        return [f"Missing FTL keys in crates/server/src/i18n.rs: {', '.join(missing[:10])}"]
    return []


def main() -> int:
    issues: list[str] = []
    issues.extend(check_no_target_dir())
    for p in sorted(glob.glob("crates/**/*.rs", recursive=True)):
        raw = open(p, encoding="utf-8").read()
        masked = strip_comments_and_strings(raw)
        issues.extend(check_brace_balance(p, masked))
        issues.extend(check_iterator_move_captures(p, raw, masked))
    issues.extend(check_ftl_keys())
    if issues:
        print("AUDIT FAILED:")
        for item in issues:
            print(" -", item)
        return 1
    print("AUDIT PASSED: braces, iterator-closure captures, i18n keys, and workspace boundaries clean.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
