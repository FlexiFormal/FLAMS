#!/usr/bin/env bash

script_name="${0##*/}"
usage="Usage: $script_name -i <input_file> -js <js_file> -o <output_file>";

while [[ $# -gt 0 ]]; do
    key="$1"
    case $key in
        -i|--input)
            input_file="$2"
            shift
            shift
            ;;
        -js|--js)
            js_file="$2"
            shift
            shift
            ;;
        -o|--output)
            output_file="$2"
            shift
            shift
            ;;
        *)
            printf "Unknown option: %s\n%s" "$1" "$usage";
            exit 1
            ;;
    esac
done

if [ -z "$input_file" ] || [ -z "$js_file" ] || [ -z "$output_file" ]; then
    printf "Usage: %s\n" "$usage";
    exit 1
fi


file_contents_b64=$(base64 -i "$input_file" -w 0);
js_contents=$(cat "$js_file");

printf "input size: %d\ncompressed size: %d\n" "${#file_contents_b64}" "${#file_contents_b64_compressed}";

# printf "Result: $file_contents_b64";

patch=$(cat <<EOF
async function decode(base64) {
  const data = 'data:application/octet-stream;base64,' + base64;

  const {body} = await fetch(data);

  const response = new Response(body, {headers: {"content-type": "application/octet-stream"}});

  return await response.arrayBuffer();
}

async function init() {
  if (wasm !== undefined) return wasm;

  const imports = __wbg_get_imports();

  __wbg_init_memory(imports);

  const buffer = await decode('$file_contents_b64');

  const module = new WebAssembly.Module(buffer);

  const instance = new WebAssembly.Instance(module, imports);

  return __wbg_finalize_init(instance, module);
}

const initAsync = __wbg_init;

export {initAsync, initSync, init};
EOF
);

js_contents_patched="${js_contents}";
js_contents_patched=$(echo "$js_contents_patched" | sed "s|export { initSync };||");
js_contents_patched=$(echo "$js_contents_patched" | sed "s|export default __wbg_init;||");
js_contents_patched="${js_contents_patched}${patch}";

echo "${js_contents_patched}" | \
esbuild \
  --bundle \
  --minify \
  --format=esm \
  --target=es2020 \
  --platform=browser \
  --tree-shaking=true \
  --analyze \
  --outfile="${output_file}";
