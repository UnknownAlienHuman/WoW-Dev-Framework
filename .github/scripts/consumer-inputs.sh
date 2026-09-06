#!/usr/bin/env bash
# CI-only acquisition of two named upstream test consumers, never a product updater.
set -euo pipefail
root="$RUNNER_TEMP/consumer-inputs"
mkdir -p "$root/emmy" "$root/luals"
case "$RUNNER_OS" in
  Linux) emmy_name=emmylua_check-linux-x64-glibc.2.17.tar.gz; suffix=linux-x64.tar.gz; extension= ;;
  Windows) emmy_name=emmylua_check-win32-x64.zip; suffix=win32-x64.zip; extension=.exe ;;
  *) echo 'Unsupported consumer probe platform' >&2; exit 2 ;;
esac
for consumer in emmy luals; do
  case "$consumer" in
    emmy) repo=EmmyLuaLs/emmylua-analyzer-rust ;;
    luals) repo=LuaLS/lua-language-server ;;
  esac
  # Resolve once. The exact release/asset/digest is evidence, not a permanent pin.
  gh api "repos/$repo/releases/latest" > "$root/$consumer-release.json"
  tag="$(jq -er '.tag_name' "$root/$consumer-release.json")"
  [[ "$tag" =~ ^[A-Za-z0-9._-]+$ ]]
  jq -e '.draft == false and .prerelease == false' "$root/$consumer-release.json" >/dev/null
  if [[ "$consumer" == emmy ]]; then name="$emmy_name"; else name="lua-language-server-$tag-$suffix"; fi
  jq -e --arg name "$name" '[.assets[] | select(.name == $name)] | if length == 1 then .[0] else error("asset selection is not unique") end' \
    "$root/$consumer-release.json" > "$root/$consumer-asset.json"
  digest="$(jq -er '.digest' "$root/$consumer-asset.json")"
  url="$(jq -er '.browser_download_url' "$root/$consumer-asset.json")"
  [[ "$digest" =~ ^sha256:[0-9a-f]{64}$ ]]
  [[ "$url" == "https://github.com/$repo/releases/download/"* ]]
  curl --fail --silent --show-error --location --proto '=https' --proto-redir '=https' \
    --max-time 120 --max-filesize 67108864 "$url" -o "$root/$consumer.archive"
  printf '%s  %s\n' "${digest#sha256:}" "$root/$consumer.archive" | sha256sum --check --status
  case "$RUNNER_OS" in
    Linux) tar -xzf "$root/$consumer.archive" -C "$root/$consumer" ;;
    Windows) unzip -q "$root/$consumer.archive" -d "$root/$consumer" ;;
  esac
  # Exact packaged entrypoints; a changed upstream layout fails, never selects by search order.
  if [[ "$consumer" == emmy ]]; then binary="$root/emmy/emmylua_check$extension"; key=WDF_EMMY_CHECK;
  else binary="$root/luals/bin/lua-language-server$extension"; key=WDF_LUALS; fi
  [[ -f "$binary" && ! -L "$binary" ]]
  chmod +x "$binary"
  # Hash stdin in binary mode: filename escaping must not prefix the digest on Windows.
  executable_digest="sha256:$(sha256sum --binary < "$binary" | cut -d' ' -f1)"
  [[ "$executable_digest" =~ ^sha256:[0-9a-f]{64}$ ]]
  if [[ "$RUNNER_OS" == Windows ]]; then binary="$(cygpath -m "$binary")"; fi
  printf '%s=%s\n%s_SHA256=%s\n' "$key" "$binary" "$key" "$executable_digest" >> "$GITHUB_ENV"
done
