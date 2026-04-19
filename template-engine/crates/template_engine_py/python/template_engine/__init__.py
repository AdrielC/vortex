"""Thin Python wrappers around template_engine JSON-in/out APIs."""
from __future__ import annotations

from template_engine_py import bind_compiled_json, compile_template_json, render_compiled_json

__all__ = [
    "bind_compiled_json",
    "compile_template_json",
    "render_compiled_json",
]
