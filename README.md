# zp

The "zp" command is a custom command that takes one argument, which is the name of the source file. The purpose of this command is to copy the contents of the source file to the clipboard, allowing users to easily paste the contents into another file or program.

To use the "zp" command, simply open your terminal or command prompt and type "zp" followed by the name of the source file. For example:

```
zp myFile.txt

```

This will copy the contents of "myFile.txt" to the clipboard.

The "zp" command is particularly useful for quickly copying text or data from one file to another without having to manually select and copy the text. This can save time and effort, especially when working with large or complex files.

## Install

It's best to use rustup to get setup with a Rust toolchain, then you can run:

`cargo install atuin`

### Homebrew
`brew install atuin`

### From source
```
git clone https://github.com/ellie/atuin.git
cd atuin
cargo install --path .
```

