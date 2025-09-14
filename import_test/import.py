#!/usr/bin/env python3
"""
Import 200 random points from a large geo file into the API.

Supported inputs (auto-detected, or override with --format):
 - CSV (any delimiter; header preferred). Looks for lat/lon columns with common aliases.
 - JSON Lines (one JSON object per line).
 - Plain text: space/comma-separated numeric columns (lat lon [alt spd azm]).

Usage (defaults shown):
  python import_test/import.py \
	--file import_test/geo_locations_astana_hackathon \
	--endpoint http://127.0.0.1:8080/api/points \
	--sample-size 200
"""

from __future__ import annotations

import argparse
import csv
import io
import json
import os
import random
import sys
import time
import urllib.error
import urllib.request
from dataclasses import dataclass
from typing import Dict, Iterable, Iterator, List, Optional, Tuple


# --- Data mapping helpers ----------------------------------------------------

LAT_KEYS = ["lat", "latitude", "y"]
LON_KEYS = ["lon", "long", "lng", "longitude", "x"]
ALT_KEYS = ["alt", "altitude", "elev", "elevation", "z"]
SPD_KEYS = ["spd", "speed", "velocity", "v"]
AZM_KEYS = ["azm", "azimuth", "heading", "bearing", "course"]
ID_KEYS = ["randomized_id", "random_id", "id", "track_id", "device_id"]


def _find_key(d: Dict[str, object], candidates: List[str]) -> Optional[str]:
	lower = {k.lower(): k for k in d.keys()}
	for c in candidates:
		if c in lower:
			return lower[c]
	return None


def _to_float(v: object, default: float = 0.0) -> float:
	if v is None:
		return default
	try:
		if isinstance(v, (int, float)):
			return float(v)
		s = str(v).strip().replace(",", ".")
		return float(s)
	except Exception:
		return default


def _gen_random_id() -> int:
	# 63-bit positive integer
	return random.getrandbits(63)


@dataclass
class Point:
	randomized_id: int
	lat: float
	lon: float
	alt: float = 0.0
	spd: float = 0.0
	azm: float = 0.0

	def to_dict(self) -> Dict[str, object]:
		return {
			"randomized_id": int(self.randomized_id),
			"lat": float(self.lat),
			"lon": float(self.lon),
			"alt": float(self.alt),
			"spd": float(self.spd),
			"azm": float(self.azm),
		}


# --- Parsers -----------------------------------------------------------------

def iter_json_lines(path: str) -> Iterator[Dict[str, object]]:
	with open(path, "r", encoding="utf-8", errors="ignore") as f:
		for line in f:
			line = line.strip()
			if not line:
				continue
			# Allow trailing commas
			if line.endswith(","):
				line = line[:-1]
			if not (line.startswith("{") and line.endswith("}")):
				# Not a JSON object line
				continue
			try:
				yield json.loads(line)
			except json.JSONDecodeError:
				continue


def iter_csv_rows(path: str) -> Iterator[Dict[str, object]]:
	with open(path, "r", encoding="utf-8", errors="ignore", newline="") as f:
		# Sniff dialect from a sample
		sample = f.read(4096)
		f.seek(0)
		try:
			dialect = csv.Sniffer().sniff(sample, delimiters=",;\t| ")
		except Exception:
			dialect = csv.excel
		# Try detect header
		try:
			has_header = csv.Sniffer().has_header(sample)
		except Exception:
			has_header = True
		reader: csv.reader | csv.DictReader
		if has_header:
			reader = csv.DictReader(f, dialect=dialect)
			for row in reader:
				# Normalize to plain dict[str, object]
				yield {k.strip(): (v.strip() if isinstance(v, str) else v) for k, v in row.items() if k is not None}
		else:
			reader = csv.reader(f, dialect=dialect)
			for fields in reader:
				# Heuristic positional mapping: lat,lon,(opt alt, spd, azm)
				if not fields:
					continue
				vals = [s.strip() for s in fields if s is not None]
				row: Dict[str, object] = {}
				if len(vals) >= 2:
					row["lat"] = vals[0]
					row["lng"] = vals[1]
				if len(vals) >= 3:
					row["alt"] = vals[2]
				if len(vals) >= 4:
					row["spd"] = vals[3]
				if len(vals) >= 5:
					row["azm"] = vals[4]
				yield row


def iter_plain_numeric(path: str) -> Iterator[Dict[str, object]]:
	with open(path, "r", encoding="utf-8", errors="ignore") as f:
		for line in f:
			line = line.strip()
			if not line:
				continue
			# Split by comma or whitespace
			tokens = [t for t in line.replace("\t", " ").replace(";", ",").replace("|", ",").replace(" ", ",").split(",") if t]
			if len(tokens) < 2:
				continue
			row: Dict[str, object] = {
				"lat": tokens[0],
				"lng": tokens[1],
			}
			if len(tokens) >= 3:
				row["alt"] = tokens[2]
			if len(tokens) >= 4:
				row["spd"] = tokens[3]
			if len(tokens) >= 5:
				row["azm"] = tokens[4]
			yield row


def sniff_format(path: str) -> str:
	# Returns one of: 'jsonl', 'csv', 'plain'
	try:
		with open(path, "r", encoding="utf-8", errors="ignore") as f:
			head = f.read(8192).lstrip()
	except Exception:
		return "plain"
	if head.startswith("{"):
		return "jsonl"  # likely JSON lines without wrapping array
	if head.startswith("["):
		# A proper JSON array. We don't stream this efficiently without extra deps,
		# but often it's one-object-per-line array; treat as jsonl best-effort.
		return "jsonl"
	# Try a quick CSV check: commas/semicolons and no braces
	if ("," in head or ";" in head or "\t" in head or "|" in head) and "{" not in head:
		return "csv"
	return "plain"


# --- Transform row -> Point --------------------------------------------------

def row_to_point(row: Dict[str, object]) -> Optional[Point]:
	# Choose source keys
	lat_k = _find_key(row, LAT_KEYS)
	lon_k = _find_key(row, LON_KEYS)
	if not lat_k or not lon_k:
		return None
	alt_k = _find_key(row, ALT_KEYS)
	spd_k = _find_key(row, SPD_KEYS)
	azm_k = _find_key(row, AZM_KEYS)
	id_k = _find_key(row, ID_KEYS)

	lat = _to_float(row.get(lat_k))
	lng = _to_float(row.get(lon_k))
	alt = _to_float(row.get(alt_k), 0.0) if alt_k else 0.0
	spd = _to_float(row.get(spd_k), 0.0) if spd_k else 0.0
	azm = _to_float(row.get(azm_k), 0.0) if azm_k else 0.0

	# Basic sanity
	if not (-90.0 <= lat <= 90.0) or not (-180.0 <= lng <= 180.0):
		return None

	rid = row.get(id_k) if id_k else None
	try:
		rid_int = int(rid) if rid is not None and str(rid).strip() != "" else _gen_random_id()
	except Exception:
		rid_int = _gen_random_id()

	return Point(randomized_id=rid_int, lat=lat, lon=lng, alt=alt, spd=spd, azm=azm)


# --- Reservoir sampling ------------------------------------------------------

def sample_stream(rows: Iterable[Dict[str, object]], k: int) -> List[Point]:
	reservoir: List[Point] = []
	n = 0
	for row in rows:
		pt = row_to_point(row)
		if pt is None:
			continue
		n += 1
		if len(reservoir) < k:
			reservoir.append(pt)
		else:
			j = random.randrange(n)
			if j < k:
				reservoir[j] = pt
	return reservoir


# --- HTTP client (stdlib) ----------------------------------------------------

def post_points(endpoint: str, points: List[Point], timeout: float = 30.0) -> Tuple[int, str]:
	payload = {"points": [p.to_dict() for p in points]}
	data = json.dumps(payload).encode("utf-8")
	req = urllib.request.Request(endpoint, data=data, method="POST", headers={
		"Content-Type": "application/json",
		"Accept": "application/json",
	})
	try:
		with urllib.request.urlopen(req, timeout=timeout) as resp:
			status = getattr(resp, "status", 200)
			body = resp.read().decode("utf-8", errors="ignore")
			return status, body
	except urllib.error.HTTPError as e:
		try:
			body = e.read().decode("utf-8", errors="ignore")
		except Exception:
			body = str(e)
		return e.code, body
	except Exception as e:
		return 0, str(e)


# --- CLI ---------------------------------------------------------------------

def main(argv: Optional[List[str]] = None) -> int:
	ap = argparse.ArgumentParser(description="Import random points into API")
	ap.add_argument("--file", default="geo_locations_astana_hackathon.csv", help="Input file path")
	ap.add_argument("--format", choices=["auto", "csv", "jsonl", "plain"], default="auto", help="Input format override")
	ap.add_argument("--endpoint", default="http://127.0.0.1:8080/api/points", help="API endpoint (POST)")
	ap.add_argument("--sample-size", type=int, default=200, help="Number of points to sample")
	ap.add_argument("--seed", type=int, default=None, help="Random seed for reproducibility")
	args = ap.parse_args(argv)

	if args.seed is not None:
		random.seed(args.seed)
	else:
		random.seed(int(time.time() * 1000) ^ os.getpid())

	if not os.path.exists(args.file):
		print(f"Input file not found: {args.file}", file=sys.stderr)
		return 2

	fmt = sniff_format(args.file) if args.format == "auto" else args.format

	if fmt == "jsonl":
		rows = iter_json_lines(args.file)
	elif fmt == "csv":
		rows = iter_csv_rows(args.file)
	else:
		rows = iter_plain_numeric(args.file)

	points = sample_stream(rows, args.sample_size)

	if not points:
		print("No valid points parsed from input.", file=sys.stderr)
		return 3

	print(f"Parsed and sampled {len(points)} points. Posting to {args.endpoint} …")
	status, body = post_points(args.endpoint, points)
	if status == 200:
		print("Success: server accepted points.")
		return 0
	else:
		print(f"Request failed (status={status}). Body:\n{body}", file=sys.stderr)
		return 4


if __name__ == "__main__":
	raise SystemExit(main())
