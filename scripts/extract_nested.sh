#!/usr/bin/env bash
# Extract every nested .zip/.rar/.7z under ROOT into a sibling folder named after the archive.
# Repeats until no new archives appear (archives inside archives). Original archives are kept.
# Usage: extract_nested.sh ROOT [LOGFILE]
set -u
ROOT="${1:?usage: extract_nested.sh ROOT [LOGFILE]}"
LOG="${2:-$ROOT/_nested_extract_log.txt}"
DONE="$LOG.done"
touch "$DONE"
SEVENZ="$(command -v 7zz || command -v 7z)" || { echo "7-Zip (7zz) not found: brew install sevenzip"; exit 1; }

for round in 1 2 3 4; do
  new=0
  while IFS= read -r -d '' a; do
    grep -Fxq -- "$a" "$DONE" && continue
    base="${a%.*}"
    dest="$base"
    # don't merge into an unrelated existing folder of the same name
    if [ -e "$dest" ] && [ -n "$(ls -A "$dest" 2>/dev/null)" ]; then dest="$base (unpacked)"; fi
    mkdir -p "$dest"
    # -p with a dummy password: encrypted archives fail fast instead of prompting
    if "$SEVENZ" x -y -bso0 -bsp0 -pNOPASSWORD -o"$dest" -- "$a" </dev/null >>"$LOG.7z.txt" 2>&1; then
      echo "OK   $a" >>"$LOG"
    elif command -v unar >/dev/null && unar -q -f -D -password NOPASSWORD -o "$dest" -- "$a" </dev/null >>"$LOG.7z.txt" 2>&1; then
      # Homebrew's 7zz has no RAR codec: fall back to unar (brew install unar)
      echo "OK   $a" >>"$LOG"
    else
      echo "FAIL $a" >>"$LOG"
    fi
    # never keep symlinks from archives (could point outside the library)
    find "$dest" -type l -delete 2>/dev/null
    rmdir "$dest" 2>/dev/null
    echo "$a" >>"$DONE"
    new=$((new + 1))
  done < <(find "$ROOT" -type f \( -iname '*.zip' -o -iname '*.rar' -o -iname '*.7z' \) \
             -not -path '*/lasercut-browser/*' -print0)
  echo "round $round: extracted $new archives"
  [ "$new" -eq 0 ] && break
done
echo "OK: $(grep -c '^OK' "$LOG")  FAIL: $(grep -c '^FAIL' "$LOG")  (log: $LOG)"
