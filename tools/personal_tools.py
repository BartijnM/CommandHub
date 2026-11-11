"""Read-only utilities for the CommandHub data folder."""

import csv
import hashlib
import http.client
import io
import json
import os
from pathlib import Path
import re
import shutil
import stat
import subprocess
import sys
from datetime import datetime, timezone


TEXT_TYPES = {".txt", ".md", ".log", ".json", ".csv"}
NOTE_TYPES = {".txt", ".md"}
IMAGE_TYPES = {".png", ".jpg", ".jpeg", ".gif", ".webp", ".bmp", ".tif", ".tiff"}
VIDEO_TYPES = {".mp4", ".mov", ".m4v", ".mkv", ".webm", ".avi", ".mpeg", ".mpg"}
IMAGE_FORMATS = "png_pipe,jpeg_pipe,gif,webp_pipe,bmp_pipe,tiff_pipe"
VIDEO_FORMATS = "mov,matroska,webm,avi,mpeg"
SERVICES = {"commandhub": (3000, "/status"), "local_web": (80, "/")}
MAX_TEXT = 1024 * 1024
MAX_ENTRIES = 10_000
MAX_MATCHES = 25


class ToolError(Exception):
    pass


def data_root():
    configured = os.environ.get("COMMANDHUB_DATA_DIR")
    path = Path(configured) if configured else Path.home() / "CommandHub-data"
    if not path.is_absolute() or not path.is_dir():
        raise ToolError("Maak CommandHub-data aan in je thuismap of stel COMMANDHUB_DATA_DIR in op een bestaande absolute map.")
    return path.resolve(strict=True)


def data_path(root, value, *, directory=False):
    relative = Path(value)
    if not value or relative.anchor or ".." in relative.parts or ":" in value:
        raise ToolError("Gebruik een relatief pad binnen de gegevensmap, zonder '..'.")
    try:
        path = (root / relative).resolve(strict=True)
    except (OSError, RuntimeError):
        raise ToolError("Het opgegeven pad bestaat niet of is niet toegankelijk.") from None
    if not path.is_relative_to(root):
        raise ToolError("Het pad ligt buiten de gegevensmap.")
    if directory and not path.is_dir():
        raise ToolError("Het opgegeven pad moet een map zijn.")
    if not directory and not path.is_file():
        raise ToolError("Het opgegeven pad moet een normaal bestand zijn.")
    return path


def entries(directory, *, recursive=False):
    """Return regular entries without following links or Windows junctions."""
    pending = [directory]
    visited = 0
    while pending:
        with os.scandir(pending.pop()) as children:
            for child in children:
                visited += 1
                if visited > MAX_ENTRIES:
                    raise ToolError("De map bevat te veel items. Kies een kleinere map.")
                try:
                    info = child.stat(follow_symlinks=False)
                except FileNotFoundError:
                    continue
                attributes = getattr(info, "st_file_attributes", 0)
                if stat.S_ISLNK(info.st_mode) or attributes & stat.FILE_ATTRIBUTE_REPARSE_POINT:
                    continue
                path = Path(child.path)
                if stat.S_ISREG(info.st_mode):
                    yield path, info
                elif recursive and stat.S_ISDIR(info.st_mode):
                    pending.append(path)


def overview(root, directory, *, suffixes=None, recursive=False):
    files = [
        (path, info) for path, info in entries(directory, recursive=recursive)
        if suffixes is None or path.suffix.lower() in suffixes
    ]
    files.sort(key=lambda entry: (-entry[1].st_mtime_ns, str(entry[0])))
    return {
        "files": [
            {
                "path": path.relative_to(root).as_posix(),
                "size_bytes": info.st_size,
                "modified_utc": datetime.fromtimestamp(info.st_mtime, timezone.utc).isoformat(),
            }
            for path, info in files[:25]
        ],
        "truncated": len(files) > 25,
    }


def read_text(path):
    with path.open("rb") as stream:
        data = stream.read(MAX_TEXT + 1)
    if len(data) > MAX_TEXT:
        raise ToolError("Het tekstbestand is groter dan 1 MiB.")
    try:
        return data.decode("utf-8-sig")
    except UnicodeDecodeError:
        raise ToolError("Dit bestand is geen UTF-8-tekst.") from None


def search(root, directory, query, suffixes):
    if not query.strip() or len(query) > 256 or "\n" in query or "\r" in query:
        raise ToolError("Gebruik een zoekterm van maximaal 256 tekens op één regel.")
    pattern = re.compile(re.escape(query), re.IGNORECASE)
    matches = []
    skipped = 0
    inspected = 0
    for path, info in entries(directory, recursive=True):
        if path.suffix.lower() not in suffixes:
            continue
        if inspected >= 500:
            return {"matches": matches, "skipped_files": skipped, "truncated": True}
        inspected += 1
        if info.st_size > MAX_TEXT:
            skipped += 1
            continue
        try:
            lines = read_text(path).splitlines()
        except (OSError, ToolError):
            skipped += 1
            continue
        for number, line in enumerate(lines, start=1):
            match = pattern.search(line)
            if match is None:
                continue
            if len(matches) == MAX_MATCHES:
                return {"matches": matches, "skipped_files": skipped, "truncated": True}
            start = max(0, match.start() - 60)
            matches.append({
                "path": path.relative_to(root).as_posix(),
                "line": number,
                "text": line[start:start + 340],
            })
    return {"matches": matches, "skipped_files": skipped, "truncated": False}


def file_sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(MAX_TEXT), b""):
            digest.update(chunk)
    return {"sha256": digest.hexdigest()}


def csv_preview(path):
    check_suffix(path, {".csv", ".tsv"})
    content = read_text(path)
    try:
        delimiter = csv.Sniffer().sniff(content[:4096], delimiters=",;\t").delimiter
    except csv.Error:
        delimiter = "\t" if path.suffix.lower() == ".tsv" else ","
    rows = csv.reader(io.StringIO(content, newline=""), delimiter=delimiter, strict=True)
    header = next(rows, [])
    preview = [row for _, row in zip(range(11), rows)]
    visible = [header, *preview[:10]]
    truncated = len(preview) > 10 or any(
        len(row) > 40 or any(len(cell) > 160 for cell in row[:40]) for row in visible
    )
    clipped = [[cell[:160] for cell in row[:40]] for row in visible]
    return {"delimiter": delimiter, "columns": clipped[0], "rows": clipped[1:], "truncated": truncated}


def check_suffix(path, allowed):
    if path.suffix.lower() not in allowed:
        raise ToolError("Dit bestandsformaat wordt door deze tool niet ondersteund.")


def inspect_file(path, program, arguments):
    executable = shutil.which(program)
    if executable is None or (os.name == "nt" and Path(executable).suffix.lower() != ".exe"):
        raise ToolError(f"Installeer {program} en zet het op het PATH van CommandHub.")
    with path.open("rb") as stream:
        result = subprocess.run(
            [executable, *arguments],
            stdin=stream,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            check=False,
            timeout=10,
        )
    if result.returncode:
        raise ToolError(f"{program} kon dit bestand niet uitlezen.")
    return result.stdout


def pdf_info(path):
    check_suffix(path, {".pdf"})
    output = inspect_file(path, "pdfinfo", ["-enc", "UTF-8", "-"])
    metadata = {}
    for line in output.decode("utf-8", errors="replace").splitlines():
        key, separator, value = line.partition(":")
        if separator and len(metadata) < 32:
            metadata[key[:64]] = value.strip()[:300]
    return {"metadata": metadata}


def media_info(path, *, image):
    check_suffix(path, IMAGE_TYPES if image else VIDEO_TYPES)
    output = inspect_file(path, "ffprobe", [
        "-v", "error",
        "-protocol_whitelist", "pipe",
        "-format_whitelist", IMAGE_FORMATS if image else VIDEO_FORMATS,
        "-select_streams", "v:0",
        "-show_entries", "stream=codec_name,width,height:format=format_name,duration",
        "-of", "json",
        "-i", "pipe:0",
    ])
    result = json.loads(output)
    streams = result.get("streams", [])
    if not streams:
        raise ToolError("Er is geen beeldspoor gevonden.")
    return {"stream": streams[0], "format": result.get("format", {})}


def disk_space():
    if os.name == "nt":
        import ctypes
        from string import ascii_uppercase

        kernel = ctypes.windll.kernel32
        kernel.GetDriveTypeW.argtypes = [ctypes.c_wchar_p]
        mask = kernel.GetLogicalDrives()
        paths = [
            Path(f"{letter}:\\") for index, letter in enumerate(ascii_uppercase)
            if mask & (1 << index) and kernel.GetDriveTypeW(f"{letter}:\\") in {2, 3}
        ]
    else:
        paths = [Path("/")]
        volumes = Path("/Volumes")
        if volumes.is_dir():
            paths.extend(path for path in volumes.iterdir() if not path.is_symlink() and path.is_mount())
        home = Path.home()
        if home.stat().st_dev != paths[0].stat().st_dev:
            paths.append(home)
    disks = []
    for path in paths:
        try:
            usage = shutil.disk_usage(path)
        except OSError:
            continue
        disks.append({"path": str(path), "total_bytes": usage.total, "free_bytes": usage.free})
    return {"volumes": disks}


def service_health(name):
    if name not in SERVICES:
        raise ToolError("Kies commandhub of local_web.")
    port, route = SERVICES[name]
    connection = http.client.HTTPConnection("127.0.0.1", port, timeout=2)
    try:
        connection.request("GET", route)
        status = connection.getresponse().status
        return {"service": name, "reachable": True, "healthy": 200 <= status < 300, "status": status}
    except (OSError, http.client.HTTPException):
        return {"service": name, "reachable": False, "healthy": False, "status": None}
    finally:
        connection.close()


def run(action, arguments):
    if action == "disk_space":
        return disk_space()
    if action == "service_health":
        return service_health(arguments[0])

    root = data_root()
    if action == "folder_overview":
        return overview(root, data_path(root, arguments[0], directory=True))
    if action == "document_search":
        return search(root, root, arguments[0], TEXT_TYPES)
    if action in {"notes_search", "notes_recent"}:
        notes = data_path(root, "notes", directory=True)
        if action == "notes_search":
            return search(root, notes, arguments[0], NOTE_TYPES)
        return overview(root, notes, suffixes=NOTE_TYPES, recursive=True)

    path = data_path(root, arguments[0])
    if action == "file_sha256":
        return file_sha256(path)
    if action == "csv_preview":
        return csv_preview(path)
    if action == "pdf_info":
        return pdf_info(path)
    return media_info(path, image=action == "image_info")


def main():
    arity = {
        "folder_overview": 1, "document_search": 1, "pdf_info": 1,
        "file_sha256": 1, "csv_preview": 1, "disk_space": 0,
        "service_health": 1, "image_info": 1, "video_info": 1,
        "notes_search": 1, "notes_recent": 0,
    }
    if len(sys.argv) < 2 or arity.get(sys.argv[1]) != len(sys.argv) - 2:
        print("Onbekende tool of onjuist aantal argumenten.", file=sys.stderr)
        return 2
    try:
        result = run(sys.argv[1], sys.argv[2:])
        print(json.dumps(result))
        return 0
    except ToolError as error:
        print(str(error), file=sys.stderr)
    except subprocess.TimeoutExpired:
        print("Het uitlezen van het bestand duurde te lang.", file=sys.stderr)
    except csv.Error:
        print("Het CSV-bestand bevat een ongeldige rij.", file=sys.stderr)
    except (OSError, ValueError, OverflowError):
        print("De gegevens konden niet worden uitgelezen.", file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main())
