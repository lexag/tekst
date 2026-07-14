import argparse
import matplotlib.pyplot as plt
import numpy as np


def read_hex_file(filename):
    values = []

    with open(filename, "r") as f:
        for line in f:
            line = line.strip()

            if line:
                values.append(int(line, 16))

    return values


def values_to_bits(values, width=None, selected_bits=None):
    """Convert values to selected bit columns."""

    if width is None:
        width = max(values).bit_length()

    if selected_bits is None:
        selected_bits = list(range(width - 1, -1, -1))

    bits = []

    for value in values:
        row = [
            (value >> bit) & 1
            for bit in selected_bits
        ]
        bits.append(row)

    return np.array(bits), selected_bits


def plot_logic_analyzer(bits, bit_numbers):
    samples, num_bits = bits.shape

    fig, ax = plt.subplots(figsize=(12, num_bits * 0.8))

    spacing = 1.5

    for index, bit_number in enumerate(bit_numbers):

        y_base = (num_bits - index - 1) * spacing

        waveform = bits[:, index] * 0.8 + y_base

        ax.step(
            range(samples),
            waveform,
            where="post",
            linewidth=2
        )

        ax.text(
            -0.5,
            y_base + 0.4,
            f"bit {bit_number}",
            ha="right",
            va="center"
        )

        ax.hlines(
            [y_base, y_base + 0.8],
            0,
            samples,
            linewidth=0.3
        )

    ax.set_xlim(0, samples)
    ax.set_ylim(-0.2, num_bits * spacing)

    ax.set_xlabel("Sample")
    ax.set_yticks([])

    ax.grid(axis="x", linestyle=":", alpha=0.5)

    plt.title("Hex Logic Analyzer")

    plt.tight_layout()
    plt.show()


def parse_bits(bit_string):
    """Convert '7,6,0' into [7,6,0]."""
    return [
        int(x)
        for x in bit_string.split(",")
    ]


def main():

    parser = argparse.ArgumentParser(
        description="Plot selected bits from hex data"
    )

    parser.add_argument(
        "file",
        help="Hex input file"
    )

    parser.add_argument(
        "-w",
        "--width",
        type=int,
        help="Force input width"
    )

    parser.add_argument(
        "-b",
        "--bits",
        help="Bits to plot, e.g. 7,6,0"
    )

    args = parser.parse_args()

    values = read_hex_file(args.file)

    selected_bits = None

    if args.bits:
        selected_bits = parse_bits(args.bits)

    bits, bit_numbers = values_to_bits(
        values,
        args.width,
        selected_bits
    )

    plot_logic_analyzer(bits, bit_numbers)


if __name__ == "__main__":
    main()
