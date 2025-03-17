# zp

`zp` is a cli command to copy the contents of the source file or of the std output buffer to the clipboard. To use the `zp`, simply open your terminal or command prompt (well install it first) and type `zp` followed by the name of the source file. For example:

```
zp my_file.txt
```

To get the first `n` words of the  file :
```
zp -s n my_file.txt
```
To get the lines between a range, i.e., to get lines from `n` till `m` of the file:
```
zp -s n -e m my_file.txt
```
Also you can use zp to copy from the std output buffer :
```
cat sample_file.txt | zp
```
This copies the entire output of the file.

You can use get a range of lines and the first n words also from the std output buffer :
```
cat sample_file.txt | zp -s 2

cat sample_file.txt | zp -s 2 -e 5
```

This gets the first 2 words and lines from 2 to 5 of the sample_file.txt respectively

This will copy the contents of "myFile.txt" to the clipboard.

### Logs and History
Every copied content is saved to a history file, allowing you to revisit previously copied entries. To view the copy history, use:

```
zp --logs
```

The "zp" command is particularly useful for quickly copying text or data from one file to another without having to manually select and copy the text. This can save time and effort, especially when working with large or complex files.

## Install

It's best to use rustup to get setup with a Rust toolchain, then you can run:

`cargo install zp`

### Homebrew
```
brew tap bahdotsh/zp
brew install zp
```

### From source
```
git clone https://github.com/bahdotsh/zp.git
cd zp
cargo install --path .
```
