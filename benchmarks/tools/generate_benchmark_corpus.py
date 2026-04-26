from pathlib import Path

ROOT = Path("benchmarks/corpus")
ROOT.mkdir(parents=True, exist_ok=True)

TEMPLATE = """
function generatedFunction{idx}(input) {{
  let result = 0;
  let temp = input;

  if (temp > 10) {{
    result = temp;
  }} else {{
    temp = temp + 1;
  }}

  for (let i = 0; i < input; i++) {{
    result = result + i;
  }}

  while (temp > 0) {{
    temp = temp - 1;
  }}

  return result;
}}

let value{idx} = generatedFunction{idx}({idx});
console.log(value{idx});
"""

def generate(path: Path, count: int) -> None:
    content = "\n".join(TEMPLATE.format(idx=i) for i in range(count))
    path.write_text(content, encoding="utf-8")

generate(ROOT / "small.js", 5)
generate(ROOT / "medium.js", 50)
generate(ROOT / "large.js", 300)

print("Generated benchmark corpus.")