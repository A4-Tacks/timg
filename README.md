# Display image for terminal (VT100)
```
A4-Tacks <wdsjxhno1001@163.com>

USAGE:
    timg [FLAGS] [OPTIONS] [FILE]

FLAGS:
        --disable-default-colors
        --disable-empty-char
    -H, --help                      Prints help information
    -s, --no-split-edge             No split edge
        --output-colors
    -V, --version                   Prints version information

OPTIONS:
    -b, --background-color <color>      Display image background color
                                        (default:000000)
        --colors <colors>               Define color output.
                                        format: 38;2;0;0;0:30,48;2;0;0;0:40
    -c, --crop-image <range>            crop image
                                        (format:-c 'x,y-w,h') (0f-100f)
        --empty-char <char>             Set empty char\n(default:\x20)
    -t, --filter <id>                   Type of filter used
                                        0: Nearest (quick)
                                        1: Triangle
                                        2: CatmullRom
                                        3: Gaussian
                                        4: Lanczos3 (default)
    -f, --foreground <char>             Set display foreground char
    -h, --height <height>               Set display image height
    -d, --optimization-level <level>    Set optimization level
                                        (default:1) [0, 255]
    -w, --width <width>                 Set display image width

ARGS:
    <FILE>    Target file
```

# Update Log

## v1.0.1
- Fixed bug: extra empty lines at the end

## v1.1.0
- Added function: output specified characters when the foreground and background colors are similar to reduce the rendering pressure and space occupation of half-height characters
- - Add Flag: --disable-empty-char
- - Add Option: --empty-char

## v1.2.0
- Add crop picture options

