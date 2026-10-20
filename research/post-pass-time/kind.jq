def kind: .prompt_head as $p |
  if ($p|test("steps 1, 3 and 4";"i")) then "1+3+4"
  elif ($p|test("steps 1 and 3";"i")) then "1+3"
  elif ($p|test("steps 3 and 4";"i")) then "3+4"
  elif ($p|test("step 3\\b";"i")) then "3"
  elif ($p|test("step 4\\b";"i")) then "4"
  elif ($p|test("^Run (a|the|purba's) .?post pass|all steps";"i")) then "full"
  else "audit" end;
