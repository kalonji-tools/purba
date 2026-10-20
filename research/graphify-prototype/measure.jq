# PROTOTYPE (#602): the cost of one step-3 run, from its subagent transcript.
# jq -s -f measure.jq agent-<id>.jsonl
def ts:
  (.timestamp | sub("\\.[0-9]+Z$"; "Z") | fromdateiso8601)
  + ((.timestamp | capture("\\.(?<ms>[0-9]+)Z$").ms // "0") | ("0." + .) | tonumber);

map(select(.timestamp != null)) as $all
| ($all | map(select(.type == "assistant" and .message.id != null))) as $a
# One API response spans several lines, one per content block; each line
# carries the same usage, so a response counts once.
| ($a | group_by(.message.id) | map(last.message.usage)) as $u
| ($a | map(.message.content[]? | select(.type == "tool_use"))) as $tools
| {
    turns: ($u | length),
    cache_read: ($u | map(.cache_read_input_tokens // 0) | add),
    cache_write: ($u | map(.cache_creation_input_tokens // 0) | add),
    fresh_input: ($u | map(.input_tokens // 0) | add),
    tool_calls: ($tools | length),
    graph_calls: ($tools
      | map(select((.input | tostring) | test("graphify|GRAPH_REPORT|graph\\.json")))
      | length),
    wall_s: ((($all | last | ts) - ($all | first | ts)) | floor)
  }
