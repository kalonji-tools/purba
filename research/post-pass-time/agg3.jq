include "kind";
def mmss: (round) as $s | "\(($s/60|floor)):\(($s%60)|tostring|if length<2 then "0"+. else . end)";
def S(f): map(f // 0) | add;
map(. + {kind: kind}) | (group_by(.kind) + [.]) | map(
 { kind: (if (map(.kind)|unique|length) > 1 then "ALL" else .[0].kind end), n: length,
   total: S(.total), gen: S(.acc.generation), find: S(.acc.find), rr: S(.acc["read-record"]), ro: S(.acc["read-other"]), gh: S(.acc.gh), oth: (S(.acc.other)), idle: S(.acc.idle),
   gaF: S(.gafter.find), gaRR: S(.gafter["read-record"]), gaRO: S(.gafter["read-other"]), gaGH: S(.gafter.gh), gaP: S(.gafter.prompt), gaO: S(.gafter.other),
   chF: S(.rchars.find), chRR: S(.rchars["read-record"]), chRO: S(.rchars["read-other"]), chGH: S(.rchars.gh),
   fresh: S(.tok_in_fresh), cread: S(.tok_in_cache_read), msgs: S(.msgs), vis: S(.vis_chars),
   cF: S(.calls.find), cRR: S(.calls["read-record"]), cRO: S(.calls["read-other"]), cGH: S(.calls.gh), mixed: S(.mixed_bash), in: S(.inherited),
   withlist: (map(select(.lists != null))|length), l12: S(.lists.l12), lread: S(.listed_read), rread: S(.records_read), l3: S(.lists.l3) })
| .[] | [.kind, .n, (.total|mmss), (.gen|mmss), (100*.gen/.total|round), (.find|mmss), (.rr|mmss), (.ro|mmss), (.gh|mmss), (.oth|mmss), (.idle|mmss),
   "|", (.gaP|mmss), (.gaF|mmss), (.gaRR|mmss), (.gaRO|mmss), (.gaGH|mmss),
   "|", .chF, .chRR, .chRO, .chGH, "|", .fresh, .cread, .msgs, .vis, "|", .cF, .cRR, .cRO, .cGH, .mixed, .in, "|", .withlist, .l12, .lread, .rread, .l3] | @tsv
