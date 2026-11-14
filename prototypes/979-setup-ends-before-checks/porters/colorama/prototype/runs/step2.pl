my $nt = 0;
while (<>) {
  if (/^\@pytest\.mark\.skipif\(os\.name != "nt"/) { $nt = 1; next; }
  if (/^\@pytest\.mark\.skipif\(os\.name == "nt"/) { next; }
  if (/^def /) { }
  s/^(\s*)assert not (.*)$/$1assert $2 is False/ or s/^(\s*)assert (?!.* is )(.*)$/$1assert $2 is True/;
  print;
  if ($nt && /^(\s*)with monkeypatch\.context\(\) as context:/) { print "$1    context.setattr(os, \"name\", \"nt\")\n"; $nt = 0; }
}
