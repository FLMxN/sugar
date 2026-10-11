# Sugar

Sugar is a blazingly fast CLI tool for retrieving storage information on Windows 10/11 machines, written in Rust.

## Features

- swift performance
- simple command-line interface
- optimized for Windows 10/11
- lightweight and easy to use

## Installation

You need Rust (and Cargo respectfully) installed on your machine.

```powershell
git clone https://github.com/FLMxN/sugar.git
cd sugar
cargo build --release
```

## Usage
```powershell
sugar <PATH: str> <TOP_K: int> <DEPTH: [-deep | -light]>
```

**PATH**: str --> absolute path to a discovery root (e.g. D:\Steam)

**TOP_K**: int --> number of samples to output (size descending)

**DEPTH**: [-deep | -light] --> sets a range of subdirs to look into (*deep* = 2-6; *light* = 2-4)

## License
No.

## Author
FLMxN / mephisto

Made in approximately 1 business day. For funsies. Or because I suck at storage management.
