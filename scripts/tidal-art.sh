#!/bin/sh
# Requires tidal-hifi (https://github.com/Mastermindzh/tidal-hifi) with the API enabled.

cd "$(dirname "$0")/.." || exit 1

TIDAL_API="http://localhost:47836"
POLL_INTERVAL=3
LAST_TRACK=""
TMP_IMG="/tmp/hm-hub-album-art.jpg"

echo "[tidal-art] Starting, polling $TIDAL_API every ${POLL_INTERVAL}s"

while true; do
	info=$(curl -sf "$TIDAL_API/current" 2>/dev/null)
	if [ -z "$info" ]; then
		echo "[tidal-art] Tidal API not responding, retrying..."
		sleep "$POLL_INTERVAL"
		continue
	fi

	track_id=$(echo "$info" | grep -o '"trackId":"[^"]*"' | cut -d'"' -f4)
	title=$(echo "$info" | grep -o '"title":"[^"]*"' | cut -d'"' -f4)
	artist=$(echo "$info" | grep -o '"artist":"[^"]*"' | cut -d'"' -f4)
	status=$(echo "$info" | grep -o '"status":"[^"]*"' | head -1 | cut -d'"' -f4)

	if [ -z "$track_id" ]; then
		echo "[tidal-art] No track ID in response"
		sleep "$POLL_INTERVAL"
		continue
	fi

	if [ "$track_id" = "$LAST_TRACK" ]; then
		sleep "$POLL_INTERVAL"
		continue
	fi

	echo "[tidal-art] Track changed: $title - $artist (id: $track_id, status: $status)"
	echo "[tidal-art] Fetching album art..."

	if ! curl -sf "$TIDAL_API/current/image" -o "$TMP_IMG" 2>/dev/null; then
		echo "[tidal-art] Failed to fetch album art"
		sleep "$POLL_INTERVAL"
		continue
	fi

	size=$(wc -c < "$TMP_IMG" | tr -d ' ')
	echo "[tidal-art] Got image: ${size} bytes, uploading..."

	if cargo run -- upload --no-crop "$TMP_IMG" 2>&1; then
		echo "[tidal-art] Upload complete"
	else
		echo "[tidal-art] Upload failed"
	fi

	LAST_TRACK="$track_id"
	sleep "$POLL_INTERVAL"
done
