#!/usr/bin/env python3
"""Generate a docker compose override from backend/application.yaml and run it."""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
from pathlib import Path
from typing import Dict, List, Tuple

try:
	import yaml
except ImportError as exc:  # pragma: no cover - dependency check
	sys.stderr.write("PyYAML is required. Install with `pip install pyyaml`\n")
	raise SystemExit(1) from exc

ROOT = Path(__file__).parent
APP_YAML = ROOT / "backend" / "application.yaml"
COMPOSE_FILE = ROOT / "docker-compose.yaml"


def load_config() -> Dict:
	if not APP_YAML.exists():
		raise FileNotFoundError(f"missing config file: {APP_YAML}")

	with APP_YAML.open("r", encoding="utf-8") as fp:
		return yaml.safe_load(fp) or {}


def select_language_profiles(cfg: Dict) -> List[str]:
	profiles: List[str] = []
	languages = cfg.get("languages", {}) or {}
	for name, entry in languages.items():
		if isinstance(entry, dict) and entry.get("enabled"):
			profiles.append(name)
	return profiles


def inject_env_from_config(cfg: Dict) -> None:
	"""Populate os.environ for compose process from application.yaml."""

	server = cfg.get("server") or {}
	if server.get("port"):
		os.environ["APP_SERVER_PORT"] = str(server["port"])

	database = cfg.get("database") or {}
	host = database.get("host") or "database"
	if host in ("localhost", "127.0.0.1"):
		host = "database"
	os.environ["APP_DATABASE_HOST"] = str(host)
	if database.get("port"):
		os.environ["APP_DATABASE_PORT"] = str(database["port"])
	if database.get("user"):
		os.environ["APP_DATABASE_USERNAME"] = str(database["user"])
	if database.get("password"):
		os.environ["APP_DATABASE_PASSWORD"] = str(database["password"])
	if database.get("database"):
		os.environ["APP_DATABASE_DATABASE"] = str(database["database"])
	if database.get("schema"):
		os.environ["APP_DATABASE_SCHEMA"] = str(database["schema"])

	os.environ["DATABASE_URL"] = (
		f"postgresql://{os.environ.get('APP_DATABASE_USERNAME','user')}:"
		f"{os.environ.get('APP_DATABASE_PASSWORD','password')}@"
		f"{os.environ.get('APP_DATABASE_HOST','database')}:"
		f"{os.environ.get('APP_DATABASE_PORT','5432')}/"
		f"{os.environ.get('APP_DATABASE_DATABASE','check_deps')}"
	)

	neo4j = cfg.get("neo4j") or {}
	uri = neo4j.get("uri") or "bolt://neo4j:7687"
	if uri.startswith("bolt://localhost") or uri.startswith("bolt://127.0.0.1"):
		uri = "bolt://neo4j:7687"
	os.environ["APP_NEO4J_URI"] = uri
	if neo4j.get("username") and neo4j.get("password"):
		os.environ["NEO4J_AUTH"] = f"{neo4j['username']}/{neo4j['password']}"
	bolt_port, http_port = _parse_neo4j_ports(uri)
	os.environ.setdefault("NEO4J_BOLT_PORT", str(bolt_port))
	os.environ.setdefault("NEO4J_HTTP_PORT", str(http_port))


def _parse_neo4j_ports(uri: str) -> Tuple[int, int]:
	# uri like bolt://host:7687
	try:
		after_scheme = uri.split("://", 1)[1]
		host_port = after_scheme.split(":", 1)[1]
		bolt_port = int(host_port)
	except Exception:
		bolt_port = 7687
	http_port = 7474
	return bolt_port, http_port


def set_build_profile(debug: bool) -> None:
	"""Set docker build profile for backend image."""
	if debug:
		os.environ["BUILD_PROFILE"] = "debug"
		os.environ.setdefault("RUST_LOG", "debug")
	else:
		os.environ.setdefault("BUILD_PROFILE", "release")


def build_command(profiles: List[str]) -> List[str]:
	cmd = ["docker", "compose"]
	for profile in profiles:
		cmd.extend(["--profile", profile])
	cmd.extend(["-f", str(COMPOSE_FILE)])
	return cmd


def main() -> int:
	parser = argparse.ArgumentParser(description="Start docker-compose based on application.yaml")
	parser.add_argument("--dry-run", action="store_true", help="Print the docker compose command and override file path without running")
	parser.add_argument("--debug", action="store_true", help="Use debug Rust build and enable debug logging")
	parser.add_argument("--build", action="store_true", help="Build images before starting containers")
	args = parser.parse_args()

	set_build_profile(args.debug)

	cfg = load_config()
	inject_env_from_config(cfg)
	profiles = select_language_profiles(cfg)

	cmd_base = build_command(profiles)

	cmd_down = cmd_base + ["down"]
	cmd_up = cmd_base + ["up"]

	if args.dry_run:
		print("Command (down):", " ".join(cmd_down))
		print("Command (up):", " ".join(cmd_up))
		return 0
	
	if args.build:
		cmd_build = cmd_base + ["build"]
		subprocess.run(cmd_build, check=True)
		
	# Clean stale networks/containers first; ignore errors
	subprocess.run(cmd_down, check=False)
	# Then bring up
	subprocess.run(cmd_up, check=True)
	return 0


if __name__ == "__main__":
	raise SystemExit(main())
