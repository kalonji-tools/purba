# jq -s --slurpfile names names.json -f analyze3.jq transcript.jsonl
def ts: (.timestamp | sub("\\.[0-9]+Z$"; "Z") | fromdateiso8601) + ((.timestamp | capture("\\.(?<ms>[0-9]+)Z$").ms // "0") | ("0." + .) | tonumber);
def txt: if type == "string" then . elif type == "array" then (map(if type=="object" then (.text // "") else tostring end) | join("\n")) else "" end;
def gh_re: "(^|[;&|(`]\\s*|\\s)gh\\s";
def recnames($s): [ $names[0][] as $n | select($s | test("(^|[^a-z0-9-])" + $n + "([^a-z0-9-]|$)")) | $n ];
def base($name; $in):
  if $name == "Read" then (if ($in.file_path // "" | test("docs/decisions/[^/]+\\.md$")) then "read-record" else "read-other" end)
  elif ($name == "Glob" or $name == "Grep") then
    (if ([$in.path, $in.pattern, $in.glob] | map(. // "") | join(" ") | test("docs/decisions")) then "find" else "read-other" end)
  elif $name == "Bash" then
    ($in.command // "") as $c |
    if ($c | test("list:records|list-records")) then "find"
    elif ($c | test("docs/decisions")) then
      (if ($c | test("\\b(cat|sed|head|tail|awk|bat)\\b")) then "read-record"
       elif ($c | test("\\b(grep|rg|ls|find|fd|wc)\\b")) then "find"
       elif ($c | test(gh_re)) then "gh"
       else "read-other" end)
    elif ($c | test(gh_re)) then "gh"
    else "read-other" end
  else "other" end;
def kinds($in):
  ($in.command // "") as $c |
  [ ($c|test("list:records|list-records")), ($c|test("docs/decisions")), ($c|test(gh_re)),
    ($c|test("\\bgit\\s|\\.github|scripts/|CONTEXT|AGENTS|\\.toml|post-pass")) ] | map(select(.)) | length;
def bn: sub(".*/"; "");

. as $all
| ($all | map(select(.timestamp != null and (.type == "user" or .type == "assistant")))) as $L
| ($L[0]) as $first
| ($first.message.content | txt) as $prompt
| ([$L[] | select(.type=="user") | (.message.content | if type=="array" then .[] else empty end) | select(.type=="tool_result")
    | {key: .tool_use_id, value: (.content | txt)}] | from_entries) as $R
| ([$L[] | select(.type=="assistant") | . as $l | (.message.content // [])[] | select(.type=="tool_use")
    | {id, name, input, ts: ($l|ts), b0: base(.name; .input)}]) as $uses0
# sequential inheritance pass
| (reduce $uses0[] as $u ({map: {}, out: []};
     ($u.input.file_path // "" | bn) as $rb
     | (if $u.name == "Read" and (.map[$rb] != null) and $u.b0 == "read-other" then .map[$rb] else
          {b: $u.b0, names: (if $u.b0 == "read-record" then
              (if $u.name == "Read" then [$u.input.file_path | bn | sub("\\.md$"; "")] else recnames($u.input.command // "") end)
             else [] end)} end) as $cls
     | .out += [$u + {b: $cls.b, names: $cls.names, inherited: ($cls.b != $u.b0), mixed: ($u.name == "Bash" and kinds($u.input) > 1)}]
     | reduce (($R[$u.id] // "") | scan("tool-results/([A-Za-z0-9_-]+\\.txt)") | .[0]) as $p (.; .map[$p] = $cls)
     | reduce (($u.input.command // "") | scan(">\\s*\"?([^\\s;&|\"]+\\.(?:txt|md|out))") | .[0] | bn) as $p (.; .map[$p] = $cls)
   ) | .out) as $uses
| ($uses | map({key: .id, value: .}) | from_entries) as $U
| ([$L[] | ts as $t |
    if .type == "assistant" then {k: "a", t: $t}
    elif (.message.content | type) == "array" and ((.message.content | map(select(.type=="tool_result")) | length) > 0)
      then (.message.content[] | select(.type=="tool_result") | {k: "r", t: $t, id: .tool_use_id, n: (.content | txt | length)})
    else {k: "u", t: $t} end] | sort_by(.t)) as $E
| (reduce range(1; $E|length) as $i (
    {pending: {}, last: "prompt", gafter: {}, rchars: {}, acc: {generation:0, find:0, "read-record":0, "read-other":0, gh:0, other:0, idle:0}};
    ($E[$i-1]) as $p | ($E[$i]) as $e | ($e.t - $p.t) as $d
    | (if $p.k == "a" then (.pending += ([$uses[] | select(.ts == $p.t) | {key: .id, value: .b}] | from_entries)) else . end)
    | if $e.k == "a" then (.acc.generation += $d | .gafter[.last] += $d)
      elif $e.k == "r" then
        (.pending | to_entries) as $pe
        | (if ($pe|length) == 0 then .acc.other += $d
           else reduce $pe[] as $x (.; .acc[$x.value] += ($d / ($pe|length))) end)
        | .last = ($U[$e.id].b // "other") | .rchars[.last] += $e.n | del(.pending[$e.id])
      else (if $i == 1 then .acc.generation += $d else .acc.idle += $d end) end
  )) as $W
| ([$L[] | select(.type=="assistant" and .message.usage != null) | {id: .message.id, u: .message.usage}]
   | group_by(.id) | map({in: (map(.u.input_tokens // 0)|max), cc: (map(.u.cache_creation_input_tokens // 0)|max),
       cr: (map(.u.cache_read_input_tokens // 0)|max), out: (map(.u.output_tokens // 0)|max)})) as $T
| ([$L[] | select(.type=="assistant") | (.message.content // [])[] |
     if .type=="text" then (.text|length) elif .type=="tool_use" then (.input|tostring|length) else 0 end] | add // 0) as $vis
| ([$prompt] + [$R[]] | map(select(test("On every pull request:"))) | .[0] // null) as $lo
| (if $lo == null then null else
    ($lo | split("May hold a stale sentence")) as $parts
    | ([$parts[0] | scan("docs/decisions/([A-Za-z0-9._-]+)\\.md") | .[0]] | unique) as $l12
    | { l12: ($l12|length), l12names: $l12,
        l3: (if ($parts|length) > 1 then ([$parts[1] | split("\n")[1:][] | scan("^- `docs/decisions/([A-Za-z0-9._-]+)\\.md:") | .[0]] | unique | length) else 0 end) }
   end) as $lists
| ([$uses[] | select(.b=="read-record") | .names[]] | unique) as $read
| {
    start: $first.timestamp,
    total: ($E[-1].t - $E[0].t),
    acc: $W.acc, gafter: $W.gafter, rchars: $W.rchars,
    calls: ($uses | group_by(.b) | map({key: .[0].b, value: length}) | from_entries),
    inherited: ($uses | map(select(.inherited)) | length),
    mixed_bash: ($uses | map(select(.mixed)) | length),
    msgs: ($T|length),
    tok_in_fresh: ($T | map(.in + .cc) | add),
    tok_in_cache_read: ($T | map(.cr) | add),
    tok_out_last: ($T[-1].out // 0),
    vis_chars: $vis,
    lists: ($lists | if . then del(.l12names) else . end),
    records_read: ($read | length),
    listed_read: (if $lists then ([$read[] | select(. as $r | $lists.l12names | index($r))] | length) else null end),
    prompt_head: ($prompt | gsub("\\s+"; " ") | .[0:110])
  }
