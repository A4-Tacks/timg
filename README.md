The image viewer on the terminal is based on the VT100 standard.

# Function
- Zoom Picture
- Rotate Picture
- Mirror image
- Inverted image
- Grayscale image
- Change interpolation algorithm
- Change background color
- Adjusting the output color difference threshold to improve output speed

# Info
crate: <https://crates.io/crates/timg>

Due to the current implementation of reading one character at a time being unavailable on Windows, it may result in the software being unavailable or not meeting expectations on Windows.
