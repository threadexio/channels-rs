from __future__ import annotations
import argparse
import struct
import io
from colorama import Fore, Style, Back


def colored(s: str, color=None, style=None) -> str:
    return f"{color or ''}{style or ''}{s}{Fore.RESET}{Back.RESET}{Style.RESET_ALL}"


def __log(prompt: str, *args):
    print(f"{prompt}:", *args)


def error(*args):
    prompt = colored("error", color=Fore.RED, style=Style.BRIGHT)
    __log(prompt, *args)


def warn(*args):
    prompt = colored("warning", color=Fore.YELLOW, style=Style.BRIGHT)
    __log(prompt, *args)


def info(*args):
    prompt = colored("info", color=Fore.GREEN, style=Style.BRIGHT)
    __log(prompt, *args)


def calculate_checksum(buf: bytes) -> int:
    s = 0

    while len(buf) >= 2:
        s += struct.unpack("H", buf[:2])[0]
        buf = buf[2:]

    if len(buf) == 1:
        s += struct.unpack("B", buf)[0]

    while s >> 16:
        s = (s >> 16) + (s & 0xFFFF)

    return (~s) & 0xFFFF


class Header:
    VERSION = 0xFD3F
    SIZE = 8

    version: int
    length: int
    checksum: int
    flags: int
    id: int

    checksum_expected: int

    raw: bytes

    def __init__(self, buf: bytes):
        if len(buf) < Header.SIZE:
            raise Exception("buffer too small")

        self.raw = buf[: Header.SIZE]
        fields = struct.unpack(">HHHBB", self.raw)

        self.version = fields[0]
        self.length = fields[1]
        self.checksum = fields[2]
        self.flags = fields[3]
        self.id = fields[4]

        self.checksum_expected = calculate_checksum(buf)


def open_file(path: str, *args, default_path: str | None = None):
    if default_path is not None and path == "-":
        path = default_path

    return open(path, *args)


class Payload:
    data: bytes | None = None

    def __init__(self, data: bytes | None):
        self.data = data

    def __len__(self) -> int:
        return len(self.data)


class Packet:
    header: Header
    payload: Payload

    def __init__(self, header: Header, payload: Payload):
        self.header = header
        self.payload = payload

    @staticmethod
    def parse(buf: bytes) -> Packet:
        h = Header(buf)
        p = Payload(buf[Header.SIZE : h.length])
        return Packet(h, p)

    def __len__(self) -> int:
        return self.header.length


def parse_bytes_to_packets(buf: bytes) -> [Packet]:
    packets = []
    while len(buf) > 0:
        packet = Packet.parse(buf)
        buf = buf[packet.header.length :]
        packets.append(packet)

    return packets


def color_code_len(l: int) -> str:
    return colored(l, Fore.YELLOW)


def color_code_file_path(path: str) -> str:
    return colored(path, Fore.BLUE, Style.BRIGHT)


def color_code_flags(is_set: bool) -> str:
    return (
        colored("set", Fore.MAGENTA, Style.BRIGHT)
        if is_set
        else colored("not set", Fore.WHITE, Style.DIM)
    )


def color_code_ok_invalid(is_ok: bool) -> str:
    return (
        colored("ok", Fore.GREEN, Style.BRIGHT)
        if is_ok
        else colored("invalid", Fore.RED, Style.BRIGHT)
    )


def color_code_const(val: str) -> str:
    return colored(val, Fore.WHITE, Style.DIM)


def analyze_main(args, raw_input: bytes):
    packets = parse_bytes_to_packets(raw_input)

    selected_packets = []
    if args.id is None:
        selected_packets = packets
    else:
        for packet_id in args.id:
            packet_id = int(packet_id)

            if packet_id >= len(packets):
                error(f"packet #{color_code_len(packet_id)} not found")
                return

            selected_packets.append(packets[packet_id])

    expected_id = 0

    print(
        f"╭─ {color_code_file_path(args.input)}: {color_code_len(len(packets))} packets - {color_code_len(len(raw_input))} bytes"
    )
    for packet in selected_packets:
        print(
            f"""│
├──● Packet: {color_code_len(len(packet))} bytes
│  ├──● Header: {color_code_len(packet.header.SIZE)} bytes
│  │  ├─○ version:  {color_code_const(f'{packet.header.version:<#10x}')}{color_code_ok_invalid(packet.header.version == packet.header.VERSION)}
│  │  ├─○ length:   {color_code_len(packet.header.length)}
│  │  ├─○ checksum: {colored(f'{packet.header.checksum:<#10x}', Fore.CYAN)}{color_code_ok_invalid(packet.header.checksum == packet.header.checksum_expected)} (expected: {color_code_const(f'{packet.header.checksum_expected:#06x}')})
│  │  ├─○ flags:    {packet.header.flags:<08b}
│  │  │             ╰──────── MORE_DATA: {color_code_flags(packet.header.flags & 0b1000_0000)}
│  │  ╰─○ id:       {packet.header.id:<#10x}{color_code_ok_invalid(packet.header.id == expected_id)}
│  ╰──● Payload: {color_code_len(len(packet.payload))} bytes
│     ╰─○ data:     [...]
""",
            end="",
        )
        expected_id += 1
    print("│\n╰⬤")


def extract_main(args, raw_input: bytes):
    def extract_packet_to(args, out: io.BytesIO, packet: Packet):
        n = 0
        if args.header:
            n += out.write(packet.header.raw)
        if args.payload:
            n += out.write(packet.payload.data)
        return n

    packets = parse_bytes_to_packets(raw_input)

    bytes_written = 0
    out_file = open_file(args.out, "wb", default_path="/dev/stdout")

    if args.id is None:
        info(f"extracting all packets ({color_code_len(len(packets))} packets)")

        for packet in packets:
            bytes_written += extract_packet_to(args, out_file, packet)
    else:
        for packet_id in args.id:
            packet_id = int(packet_id)

            if packet_id >= len(packets):
                error(f"packet #{color_code_len(packet_id)} not found")
                return

            info(f"extracting packet #{color_code_len(packet_id)}")
            bytes_written += extract_packet_to(args, out_file, packets[packet_id])

    out_file.close()
    info(
        f"writing {color_code_file_path(args.out)}: {color_code_len(bytes_written)} bytes"
    )


def cli_main(args):
    input_file = open_file(args.input, "rb", default_path="/dev/stdin")
    raw_input = input_file.read()
    input_file.close()

    subcommands = {"analyze": analyze_main, "extract": extract_main}

    subcommands[args.subcommand](args, raw_input)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "-i", "--input", help="File to dissect", required=True, metavar="<path>"
    )
    parsers = parser.add_subparsers(dest="subcommand")

    analyze_parser = parsers.add_parser(
        "analyze", help="Parse the packet(s) into a human readable tree"
    )
    analyze_parser.add_argument(
        "--id",
        help="Specify which packet to operate on (specify multiple times for many packets)",
        metavar="<index>",
        action="append",
    )

    extract_parser = parsers.add_parser(
        "extract", help="Extract parts of the packet(s)"
    )
    extract_parser.add_argument(
        "--id",
        help="Specify which packet to operate on (specify multiple times for many packets)",
        metavar="<index>",
        action="append",
    )
    extract_parser.add_argument(
        "--header", help="Extract the header(s)", action="store_true"
    )
    extract_parser.add_argument(
        "--payload", help="Extract the payload(s)", action="store_true"
    )
    extract_parser.add_argument("out", help="Specify the output file", metavar="<path>")

    args = parser.parse_args()
    cli_main(args)


if __name__ == "__main__":
    main()
