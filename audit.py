#!/usr/bin/env python3
# -*- coding: utf-8 -*-
import os
import sys
import codecs
from datetime import datetime

# Fix for Windows UTF-8 encoding
if sys.platform == "win32":
    sys.stdout = codecs.getwriter("utf-8")(sys.stdout.buffer, "strict")
    sys.stderr = codecs.getwriter("utf-8")(sys.stderr.buffer, "strict")


IGNORE_LIST = {
    ".git",
    ".gitignore",
    ".dockerignore",
    ".DS_Store",
    ".vscode",
    ".idea",
    "node_modules",
    "target",
    "dist",
    ".astro",
    "__pycache__",
    ".venv",
    ".env",
    "backend.db",
    "backend.db-shm",
    "backend.db-wal",
    "uploads",
    "archive",
    "test-results",
    "playwright-report",
    "AUDITORIA_MASTER.md",
    "1.md",
    "falta.md",
}

CODE_EXTS = {
    ".py", ".rs", ".ts", ".tsx",
    ".astro", ".svelte",
    ".js", ".mjs",
    ".sql", ".toml",
    ".css", ".scss",
    ".html", ".htm",
    ".yml", ".yaml",
    ".json",
    ".md", ".markdown",
}


def get_size_format(b, factor=1024, suffix="B"):
    """Convierte bytes a formato legible (KB, MB, etc.)"""
    for unit in ["", "K", "M", "G", "T", "P"]:
        if b < factor:
            return f"{b:.2f}{unit}{suffix}"
        b /= factor


def count_lines(file_path):
    try:
        with open(file_path, "r", encoding="utf-8", errors="ignore") as f:
            return sum(1 for line in f if line.strip())
    except Exception:
        return 0


def analyze_full_project(directory, ignore_list=None, prefix=""):
    if ignore_list is None:
        ignore_list = IGNORE_LIST

    tree_str = ""
    total_lines = 0
    total_size = 0
    layer_stats = {}  # { carpeta_top: { lines, size } }

    try:
        items = [i for i in sorted(os.listdir(directory)) if i not in ignore_list]
    except PermissionError:
        return "[Acceso Denegado]\n", 0, 0, {}

    for i, item in enumerate(items):
        path = os.path.join(directory, item)
        is_last = i == len(items) - 1
        connector = "└── " if is_last else "├── "
        child_prefix = prefix + ("    " if is_last else "│   ")

        stats = os.stat(path)

        if os.path.isdir(path):
            subtree, lines, folder_size, _ = analyze_full_project(
                path, ignore_list, child_prefix
            )
            tree_str += f"{prefix}{connector}{item}/ [{get_size_format(folder_size)}]\n"
            tree_str += subtree
            total_lines += lines
            total_size += folder_size

            # Registrar como capa top-level si es hijo directo de raíz
            if prefix == "":
                layer_stats[item] = {"lines": lines, "size": folder_size}

        else:
            file_lines = 0
            item_size = stats.st_size
            if any(item.endswith(ext) for ext in CODE_EXTS):
                file_lines = count_lines(path)
                total_lines += file_lines
            total_size += item_size

            info = f"({file_lines} LoC | {get_size_format(item_size)})"
            tree_str += f"{prefix}{connector}{item} {info}\n"

            # Archivos en raíz también como "capa"
            if prefix == "":
                layer_stats[item] = {"lines": file_lines, "size": item_size}

    return tree_str, total_lines, total_size, layer_stats


def build_layer_table(layer_stats, total_lines, total_size):
    lines_out = []
    lines_out.append("| Capa / Archivo | LoC | Peso | % LoC |")
    lines_out.append("| :--- | ---: | ---: | ---: |")

    sorted_layers = sorted(layer_stats.items(), key=lambda x: x[1]["lines"], reverse=True)

    for name, data in sorted_layers:
        pct = (data["lines"] / total_lines * 100) if total_lines > 0 else 0
        bar = "█" * int(pct / 5)  # barra visual simple
        lines_out.append(
            f"| `{name}` | {data['lines']} | {get_size_format(data['size'])} | {pct:.1f}% {bar} |"
        )

    lines_out.append(f"| **TOTAL** | **{total_lines}** | **{get_size_format(total_size)}** | 100% |")
    return "\n".join(lines_out)


def generate_audit():
    timestamp = datetime.now().strftime("%Y-%m-%d %H:%M")

    # Nombre real de la carpeta raíz
    root_name = os.path.basename(os.path.abspath("."))

    tree_content, total_lines, total_size, layer_stats = analyze_full_project(".")

    layer_table = build_layer_table(layer_stats, total_lines, total_size)

    report = (
        f"# 🛠️ Auditoría de Software — Lab 3030\n\n"
        f"> Generado: `{timestamp}`\n\n"
        f"## Resumen\n\n"
        f"| Métrica | Valor |\n"
        f"| :--- | :--- |\n"
        f"| **Proyecto** | `{root_name}` |\n"
        f"| **Líneas de Código (Netas)** | {total_lines} LoC |\n"
        f"| **Peso Total del Proyecto** | {get_size_format(total_size)} |\n"
        f"| **Timestamp** | {timestamp} |\n"
        f"| **Estado** | Activa |\n\n"
        f"## Breakdown por Capa\n\n"
        f"{layer_table}\n\n"
        f"## Mapa de Arquitectura\n\n"
        f"```text\n"
        f"{root_name}/\n"
        f"{tree_content}"
        f"```\n"
    )

    with open("AUDITORIA_MASTER.md", "w", encoding="utf-8") as f:
        f.write(report)

    print(f"✅ Auditoría completada [{timestamp}]")
    print(f"   Proyecto  : {root_name}/")
    print(f"   Total LoC : {total_lines}")
    print(f"   Peso      : {get_size_format(total_size)}")
    print(f"   Reporte   : AUDITORIA_MASTER.md")


if __name__ == "__main__":
    generate_audit()