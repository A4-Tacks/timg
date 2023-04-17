/// terminal default size.
/// (width, height)
/// note: lines is not text line, it is pixel line
pub const DEFAULT_TERM_SIZE: [SizeType; 2] = [80, 80];

use std::{
    io::{
        stdin,
        Read
    }
};

use raw_tty::IntoRawMode;
use term_lattice::{
    ScreenBuffer,
    Color,
    types::Rgb
};
use timg::{ESC, base16_to_unum, num_to_rgb};
use clap::ArgMatches;
use term_size::dimensions;
use timg::{
    get_scale,
    SizeType,
    Position,
    Float,
};
use image::{
    imageops::FilterType,
    ColorType
};


const FILTERS: &[FilterType] = &[
    FilterType::Nearest, FilterType::Triangle,
    FilterType::CatmullRom, FilterType::Gaussian,
    FilterType::Lanczos3
];

pub type Rgba = [u8; 4];


/// RGBA color to RGB color
/// # Examples
/// ```
/// use timg::rgba_to_rgb;
/// assert_eq!(rgba_to_rgb([100, 149, 237, 200], [255; 3]), [164, 195, 240]);
/// ```
pub fn rgba_to_rgb(foreground: Rgba, background: Rgb) -> Rgb {
    macro_rules! int {
        ( $x:expr ) => {
            ($x) as u8
        };
    }
    macro_rules! float {
        ( $x:expr ) => {
            ($x) as Float
        };
    }
    let [r1, g1, b1, a1] = foreground;
    let [r2, g2, b2] = background;
    let alpha = a1 as Float / 255.0;
    let [r, g, b]: [u8; 3];
    r = int!(float!(r1) * alpha + float!(r2) * (1.0 - alpha));
    g = int!(float!(g1) * alpha + float!(g2) * (1.0 - alpha));
    b = int!(float!(b1) * alpha + float!(b2) * (1.0 - alpha));
    [r, g, b]
}


#[macro_export]
macro_rules! log {
    (e:($code:expr) $( $x:expr ),* ) => {{
        log!(e $($x),* );
        ::std::process::exit($code);
    }};
    (e $( $x:expr ),* ) => (
        eprintln!( "{}[1;91m{}{0}[0m",
                   ESC, format!($($x),*) ))
}


pub fn run(matches: ArgMatches) {
    macro_rules! get_value {
        ( $name:expr ) => { matches.value_of($name) };
        ( $name:expr, $default:expr ) => {
            if let Some(value) = matches.value_of($name) {
                value
            } else {
                $default
            }
        };
    }

    let back_grounds: &[Rgb] = &get_value!("bgs", "000000,888888,ffffff")
        .split(",").map(|s| {
            if s.len() != 6 {
                log!(e:(3) "StrLenError: {:?} length is {}, need 6.", s, s.len())
            }
            num_to_rgb(base16_to_unum(s).unwrap_or_else(|| {
                log!(e:(3) "StrToHexError: {:?} is not a base16 string.", s)
            }))
        }).collect::<Vec<_>>()[..];
    let zoom_sub_ratio = {
        let s = get_value!("zoom_ratio", "0.8");
        let num: Float = s.parse().unwrap_or_else(
            |e| log!(e:(3) "StrToFloatError: {}", e));
        if num <= 0.0 || num >= 1.0 {
            log!(e:(3) "NumberOutOfRange: {} not in (0,1)", num)
        }
        num
    };
    let zoom_add_ratio = 1.0 / zoom_sub_ratio;
    let short_move_ratio = {
        let s = get_value!("short_move_ratio", "0.25");
        let num: Float = s.parse().unwrap_or_else(
            |e| log!(e:(3) "StrToFloatError: {}", e));
        if num <= 0.0 {
            log!(e:(3) "NumberOutOfRange: {} not in (0,inf)", num)
        }
        num
    };
    let long_move_ratio = {
        let s = get_value!("long_move_ratio", "0.75");
        let num: Float = s.parse().unwrap_or_else(
            |e| log!(e:(3) "StrToFloatError: {}", e));
        if num <= 0.0 {
            log!(e:(3) "NumberOutOfRange: {} not in (0,inf)", num)
        }
        num
    };
    let default_opt_level = {
        let s = get_value!("opt_level", "60");
        let num: SizeType = s.parse().unwrap_or_else(
            |e| log!(e:(3) "StrToIntError: {}", e));
        if num <= 0 {
            log!(e:(3) "NumberOutOfRange: {} not in [0,inf)", num)
        }
        num
    };
    let set_term_size: Option<Position> = {
        if let Some(size) = get_value!("term_size") {
            let nums = size.split(",")
                .map(|n| n.parse::<SizeType>()
                     .unwrap_or_else(
                         |e|
                         log!(e:(3) "StrToIntError: {}", e)))
                .collect::<Vec<_>>();
            if nums.len() != 2 {
                log!(e:(3) "need length is 2, found {}", nums.len())
            }
            Some(Position::new(nums[0], nums[1]))
        } else {
            None
        }
    };


    macro_rules! clear_screen {
        () => {
            eprint!("\x1b[2J"); // 清空屏幕
        };
    }
    let path = matches
        .value_of_os("FILE")
        .unwrap_or_else(|| {
            log!(e:(1) "GetFileError. use `-H` option print help");
        });
    let mut repr_img
        = image::open(path).unwrap_or_else(|e| {
            log!(e:(2) "ReadImageError: {}", e);
        });
    let img_size = Position::from([repr_img.width(), repr_img.height()]);
    let mut stdin = stdin().into_raw_mode().unwrap_or_else(|e| {
        log!(e:(2) "GetStdInError: {}", e);
    });
    let is_alpha: bool = match repr_img.color() {
        ColorType::L8 | ColorType::L16
            | ColorType::Rgb8 | ColorType::Rgb16
            | ColorType::Rgb32F
            => false,
        ColorType::La8
            | ColorType::La16| ColorType::Rgba8
            | ColorType::Rgba16 | ColorType::Rgba32F
            => true,
        t => log!(e:(2) "ColorTypeError: {:?}", t),
    };
    let mut is_start: bool = true;
    let mut readbuf: [u8; 1] = [0];
    'main: loop {
        let mut term_size: Position
            = Position::from(if let Some(size) = set_term_size {
                [size.x, size.y * 2]
            } else {
                match dimensions() {
                    Some(x) => [x.0 as SizeType, x.1 as SizeType * 2],
                    None => {
                        log!(e "GetTerminalSizeError. use default: {:?}",
                             DEFAULT_TERM_SIZE);
                        DEFAULT_TERM_SIZE
                    },
                }
            });
        if is_start {
            eprint!("\x1b[{}S", term_size.y >> 1); // 滚动一个屏幕, 以空出空间
        }
        clear_screen!();
        term_size.y -= 2; // 缩小终端大小一文本行以留给状态行
        let mut scale: Float = get_scale(term_size, img_size);
        let mut win_pos: Position = Position::default(); // 在图片中的绝对像素
        let mut screen_buf: ScreenBuffer
            = ScreenBuffer::new(term_size.into_array());
        screen_buf.cfg.chromatic_aberration = default_opt_level;
        let mut back_ground_color_idx = 0;
        let mut filter_idx = 4;
        let [mut grayscale, mut invert] = [false; 2];
        loop {
            screen_buf.cfg.default_color
                = Color::Rgb(back_grounds[back_ground_color_idx]);
            let scale_term_size = term_size.mul_scale(scale);
            let mut img
                = repr_img.crop_imm(win_pos.x,
                               win_pos.y,
                               scale_term_size.x,
                               scale_term_size.y)
                .resize(
                    term_size.x,
                    term_size.y,
                    FILTERS[filter_idx]);
            if invert {
                img.invert()
            }
            if grayscale {
                img = img.grayscale()
            }
            { /* flush to screen buffer */
                screen_buf.init_colors();
                let mut count: usize = 0;
                let img_width: usize = img.width() as usize;
                let line_add_idx: usize = term_size.x as usize - img_width;
                let mut i: usize = 0;
                macro_rules! flush {
                    ( $i:ident in $from:expr => $f:expr ) => {
                        for $i in $from {
                            screen_buf.set_idx(i, Color::Rgb($f));
                            i += 1;
                            count += 1;
                            if count == img_width {
                                i += line_add_idx;
                                count = 0;
                            }
                        }
                    };
                }
                if is_alpha {
                    flush!(color in img.into_rgba8().pixels()
                           => rgba_to_rgb(
                               color.0,
                               back_grounds[back_ground_color_idx]));
                } else {
                    flush!(color in img.into_rgb8().pixels() => color.0);
                }
            }
            let status_line: String = format!(concat!(
                    "\x1b[7m",
                    "ImgSize[{}x{}] ",
                    "Pos[{},{}] ",
                    "Scale[{:.2}] ",
                    "Opt[{}] ",
                    "Fl[{}] ",
                    "Help(H) ",
                    "Quit(Q)",
                    "\x1b[0m"),
                    img_size.x, img_size.y,
                    win_pos.x, win_pos.y,
                    scale,
                    screen_buf.cfg.chromatic_aberration,
                    filter_idx);
            eprint!("\x1b[H{}{}", screen_buf.flush(false), status_line);
            is_start = false;
            macro_rules! read_char {
                () => {
                    stdin.read_exact(&mut readbuf).unwrap_or_else(|e| {
                        log!(e:(2) "ReadCharError: {}", e)
                    })
                };
            }
            read_char!();
            let move_len: SizeType = {
                let num = scale.ceil() as SizeType;
                if num == 0 {
                    1
                } else {
                    num
                }
            };
            macro_rules! ctrl_err {
                ( $( $x:expr ),* ) => {
                    eprint!("\x07 \x1b[101m{}\x1b[0m",
                            format!( $( $x ),* ))
                };
            }
            let [moveb_wlen, moveb_hlen] = [
                (scale_term_size.x as Float * short_move_ratio).ceil() as SizeType,
                (scale_term_size.y as Float * short_move_ratio).ceil() as SizeType
            ];
            let [movec_wlen, movec_hlen] = [
                (scale_term_size.x as Float * long_move_ratio).ceil() as SizeType,
                (scale_term_size.y as Float * long_move_ratio).ceil() as SizeType
            ];
            match readbuf[0] as char {
                'r' => {
                    screen_buf.init_bg_colors();
                    clear_screen!();
                },
                'R' => continue 'main,
                'Q' => break, /* exit */
                'h' => {
                    let old = win_pos.x;
                    win_pos.x -= move_len;
                    if win_pos.x > old {
                        win_pos.x = 0;
                        ctrl_err!("RB");
                    }
                },
                'j' => win_pos.y += move_len,
                'k' => {
                    let old = win_pos.y;
                    win_pos.y -= move_len;
                    if win_pos.y > old {
                        win_pos.y = 0;
                        ctrl_err!("RB");
                    }
                },
                'l' => win_pos.x += move_len,

                'a' => {
                    let old = win_pos.x;
                    win_pos.x -= moveb_wlen;
                    if win_pos.x > old {
                        win_pos.x = 0;
                        ctrl_err!("RB");
                    }
                },
                's' => win_pos.y += moveb_hlen,
                'w' => {
                    let old = win_pos.y;
                    win_pos.y -= moveb_hlen;
                    if win_pos.y > old {
                        win_pos.y = 0;
                        ctrl_err!("RB");
                    }
                },
                'd' => win_pos.x += moveb_wlen,
                'A' => {
                    let old = win_pos.x;
                    win_pos.x -= movec_wlen;
                    if win_pos.x > old {
                        win_pos.x = 0;
                        ctrl_err!("RB");
                    }
                },
                'S' => win_pos.y += movec_hlen,
                'W' => {
                    let old = win_pos.y;
                    win_pos.y -= movec_hlen;
                    if win_pos.y > old {
                        win_pos.y = 0;
                        ctrl_err!("RB");
                    }
                },
                'D' => win_pos.x += movec_wlen,
                '+' | 'c' => scale *= zoom_sub_ratio,
                '-' | 'x' => scale *= zoom_add_ratio,
                'o' => { /* opt */
                    screen_buf.cfg.chromatic_aberration += 1;
                }
                'O' => { /* opt */
                    screen_buf.cfg.chromatic_aberration += 10;
                }
                'i' => { /* opt */
                    if screen_buf.cfg.chromatic_aberration != 0 {
                        screen_buf.cfg.chromatic_aberration -= 1
                    } else {
                        ctrl_err!("FV")
                    };
                }
                'I' => { /* opt */
                    if screen_buf.cfg.chromatic_aberration >= 10 {
                        screen_buf.cfg.chromatic_aberration -= 10
                    } else {
                        screen_buf.cfg.chromatic_aberration = 0;
                        ctrl_err!("FV")
                    };
                }
                'z' => {
                    back_ground_color_idx += 1;
                    back_ground_color_idx %= back_grounds.len();
                },
                'Z' => {
                    back_ground_color_idx = 0;
                },
                'f' => {
                    filter_idx += 1;
                    filter_idx %= FILTERS.len();
                },
                'g' => repr_img = repr_img.fliph(),
                'G' => repr_img = repr_img.flipv(),
                'y' => repr_img = repr_img.rotate90(),
                'Y' => repr_img = repr_img.rotate270(),
                'm' => invert = ! invert,
                'M' => grayscale = ! grayscale,
                'H' | '?' => {
                    // help
                    clear_screen!();

                    eprintln!("\x1b[H");
                    macro_rules! outlines {
                        ( $( $fmt:tt $( , $( $x:expr ),+ )? ; )* ) => {
                            $(
                                eprint!(
                                    concat!("\x1b[G", $fmt, "\n\x1b[G")
                                    $(, $( $x ),+ )?);
                            )*
                        };
                    }
                    let bgcolors_fmt = back_grounds
                        .iter().map(
                            |x|x.map(|s| format!("{:02X}", s))
                            .concat())
                        .collect::<Vec<_>>().join(",");
                    outlines!{
                        "{0}Help{0}", "-".repeat(((term_size.x - 4) >> 1) as usize);
                        "Move: move px:`hjkl`, move 1/4 term: `aswd`, move 3/4 term: `ASWD`, s/l ratio: ({:.2},{:.2})",
                            short_move_ratio, long_move_ratio;
                        "Opt: add opt: `oO`, sub opt: `iI`";
                        "Zoom: `cx` or `+-`, ratio: {:.4},{:.4}", zoom_add_ratio, zoom_sub_ratio;
                        "ReDraw: `r`";
                        "ReInit: `R`";
                        "SwitchBackground: `z` [{}]", bgcolors_fmt;
                        "InitBackground: `Z`";
                        "SetFilter: `f`, ({:?}) {:?}", FILTERS[filter_idx], FILTERS;
                        "FlipImage: `gG`";
                        "Rotate: `yY`";
                        "Invert: `m`";
                        "Grayscale: `M`";
                        "ThisHelpInfo: `H?`";
                        "Quit: `Q`";
                    };
                    eprint!("\x1b[{}H", (term_size.y >> 1) + 1);

                    read_char!();
                    clear_screen!();
                    screen_buf.init_bg_colors();
                },
                c => {
                    ctrl_err!("EI:{:?}", c)
                },
            }
            eprint!("\x1b[K"); // 清除残留状态行
        }
        break;
    }
    eprintln!("\x1b[G"); // 退出时到头部换一行
}
