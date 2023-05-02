The image viewer on the terminal is based on the xterm standard.
This is an interactive image viewer with very fast IO speed on the terminal, utilizing incremental output and other methods to significantly reduce IO data. It can perform well on low-speed terminals.

# Functions
- Zoom Picture
- Rotate Picture
- Mirror image
- Inverted image
- Grayscale image
- Change interpolation algorithm
- Change background color
- Adjusting the output color difference threshold to improve output speed


# Rendering
![example1](https://raw.githubusercontent.com/A4-Tacks/timg/main/Examples/Example1.png)
![example2](https://raw.githubusercontent.com/A4-Tacks/timg/main/Examples/Example2.png)
![example3](https://raw.githubusercontent.com/A4-Tacks/timg/main/Examples/Example3.gif)


# Info
crate: <https://crates.io/crates/timg>

Due to the current implementation of the software reading one character at a time appearing to be unavailable on Windows systems, it may result in the software being unavailable or not meeting your expectations when running on Windows systems.
