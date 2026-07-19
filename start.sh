#!/usr/bin/env bash
set -euo pipefail

repo_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
run_dir="$repo_dir/.diewan/run"
pid_file="$run_dir/hemion.pid"
log_file="$run_dir/hemion.log"
port="${HEMION_PORT:-8181}"
url="http://127.0.0.1:$port"

mkdir -p "$run_dir"
if [[ -f "$pid_file" ]] && kill -0 "$(<"$pid_file")" 2>/dev/null; then
  echo "Hemion is already running (PID $(<"$pid_file"), $url)."
  exit 0
fi
rm -f "$pid_file"
command -v dx >/dev/null || { echo "Dioxus CLI (dx) is required" >&2; exit 1; }
(cd "$repo_dir" && nohup setsid dx serve --platform web --port "$port" >>"$log_file" 2>&1 & echo $! >"$pid_file")

for _ in {1..180}; do
  if curl --fail --silent "$url" >/dev/null 2>&1; then
    echo "Hemion is ready at $url (log: $log_file)."
    exit 0
  fi
  if ! kill -0 "$(<"$pid_file")" 2>/dev/null; then
    echo "Hemion exited during startup; inspect $log_file" >&2
    exit 1
  fi
  sleep 1
done
echo "Hemion did not become ready; inspect $log_file" >&2
exit 1
