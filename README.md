# zp

The "zp" command is a custom command that takes one argument, which is the name of the source file. The purpose of this command is to copy the contents of the source file or of the std output buffer to the clipboard, allowing users to easily paste the contents into another file or program.

To use the "zp" command, simply open your terminal or command prompt and type "zp" followed by the name of the source file. For example:

```
zp my_file.txt

```

To get the first `n` (n is an integer) words of the  file : 
```
zp my_file.txt n
```
To get the lines between a range, i.e., to get lines from `n` till `m` (n and m are integers) of the file:
```
zp my_file.txt n m 
```
Also you can use zp to copy from the std output buffer : 
```
cat sample_file.txt | zp 
```
This copies the entire output of the file.

You can use get a range of lines and the first n words also from the std output buffer :
```
cat sample_file.txt | zp 2

cat sample_file.txt | zp 2 5
```

This gets the first 2 words and lines from 2 to 5 of the sample_file.txt respectively

This will copy the contents of "myFile.txt" to the clipboard.

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

