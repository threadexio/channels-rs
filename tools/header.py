from typing import Any, List

import argparse
import libscrc
import json
import subprocess
import sys


def sizeof(t: str) -> int:
    _lookup = {
        "u8": 1,
        "u16": 2,
        "u32": 4,
        "u64": 8
    }

    return _lookup[t]


def generate_fn(name: str, params: List[str], return_type: str | None, body: str, props: Any) -> str:
    if "vis" not in props:
        props["vis"] = ""

    if "unsafe" not in props:
        props["unsafe"] = False

    return f"""
{props['vis']} {'unsafe' if props['unsafe'] else ''} fn {name}({', '.join(params)}) {f'-> {return_type} ' if return_type != None else ''} {{ {body} }}
"""


def generate_header(fields) -> str:
    result = ""
    current_offset = 0
    hdr_hash_input = ""

    result += "impl Buffer {"
    for field in fields:
        hdr_hash_input += field["name"] + field["type"] + f"{current_offset}"

        if "get" in field:
            if 'map' in field['get']:
                hdr_hash_input += field["get"]["map"]

            result += generate_fn(field["get"]["fn"], ["&self"], field["type"], f"""
let x = read_offset(self.as_slice(), {current_offset});
{f'{field["get"]["map"]}(x)' if 'map' in field['get'] else 'x'}
""", field["get"])

        if "set" in field:
            if 'map' in field['set']:
                hdr_hash_input += field["set"]["map"]

            result += generate_fn(field["set"]["fn"], ["&mut self", f"value: {field['type']}"], None,
                                  f"""
write_offset(self.as_mut_slice(), {current_offset}, {f'{field["set"]["map"]}(value)' if 'map' in field['set'] else 'value'});
""", field["set"])

        current_offset += sizeof(field["type"])

    # Realistically any other 16-bit hash
    # works here just as well
    hdr_hash = libscrc.modbus(hdr_hash_input.encode())

    result += f"""
pub const HEADER_HASH: u16 = {hex(hdr_hash)};

pub const HEADER_SIZE: usize = {current_offset};
    """
    result += "}"
    return result


def format_code(code: str) -> str:
    child = subprocess.Popen(
        ["rustfmt", "--emit=stdout"], stdin=subprocess.PIPE, stdout=subprocess.PIPE)
    child.stdin.write(code.encode())
    stdout, _ = child.communicate()
    return stdout.decode("utf-8")


def main():
    parser = argparse.ArgumentParser(
        description="Generate header code for channels-rs"
    )
    parser.add_argument(
        "-o", "--output", help="Specify output filename (default: stdout)")
    parser.add_argument(
        "input", help="Specify file with header definition")

    args = parser.parse_args()

    if args.output == "-":
        args.output = None

    input_file = open(args.input, "r")

    output_file = None
    if args.output == None:
        output_file = sys.stdout
    else:
        output_file = open(args.output, "w")

    fields = json.load(input_file)
    generated = f"""
/*
 * Automatically generated from `tools/header.py`. Do not edit!
 *
 * Header spec: {args.input}
 */
{generate_header(fields)}
"""

    final = format_code(generated)
    output_file.write(final)


if __name__ == "__main__":
    main()
