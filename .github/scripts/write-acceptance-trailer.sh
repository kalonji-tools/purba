#!/usr/bin/env bash
echo "probe-890-marker: the branch's write-acceptance-trailer.sh ran"
echo "probe-890-cache-mode: event=${GITHUB_EVENT_NAME} ref=${GITHUB_REF} ACTIONS_CACHE_MODE=${ACTIONS_CACHE_MODE:-unset}"
