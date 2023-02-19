use std::{
    process::exit,
    collections::HashMap,
};
use image::{
    io::Reader as ImgReader,
    imageops::FilterType,
    DynamicImage,
    ImageError,
};
use clap::ArgMatches;
use timg::{
    image_size,
    base16_to_unum,
    num_to_rgb,
    Rgb,
    pass,
    DEFAULT_COLORS,
    ESC,
    CR,
};

#[macro_export]
macro_rules! log {
    (e:($code:expr) $( $x:expr ),* ) => {
        log!(e $($x),* );
        exit($code);
    };
    (e $( $x:expr ),* ) => (
        eprintln!( "{}[1;91m{}{0}[0m",
                   ESC, format!($($x),*) ))
}

fn read_img(path: &str) -> Result<DynamicImage, ImageError> {
    ImgReader::open(path)?.decode()
}

pub fn run(matches: ArgMatches) {
    macro_rules! parse {
        ( $x:expr ) => {
            match $x.parse() {
                Ok(n) => n,
                Err(_) => {
                    log!(e:(2)
                         "ParseError: {:?}", $x);
                },
            }
        };
    }
    #[doc = r#"
    $name
    $default_value
    $type_to_target_macro"#]

    macro_rules! get_val {
        ( $name:expr , $default:expr , $macro:tt )
            => (match matches.value_of($name) {
                    None => $default,
                    Some(w) => $macro!(w)});}
    macro_rules! get_filter {
        ( $id:expr ) => (match $id {
            "0" => FilterType::Nearest,
            "1" => FilterType::Triangle,
            "2" => FilterType::CatmullRom,
            "3" => FilterType::Gaussian,
            "4" => FilterType::Lanczos3,
            &_ => {
                log!(e:(2)
                     "FilterTypeError: {}",
                     $id);},
    });
    }
    // env
    let (term_width, term_height)
        : (u32, u32)
        = if let Some((w, h)) = term_size::dimensions() {
            (w as u32, h as u32 * 2)
        } else {
            (80, 40)
        };

    // get args
    let (mut width, mut height): (u32, u32)
        = (get_val!( "width", 0, parse),
        get_val!( "height", 0, parse));
    let abs_size: bool = width != 0 && height != 0;
    let has_set_size: bool = width != 0 || height != 0;
    let fg_char: &str = get_val!(
        "foreground", "▄", pass);
    let empty_char: &str = get_val!(
        "empty_char", "\x20", pass);
    let enable_empty_char: bool = ! matches.is_present("disable_empty_char");
    let split_edge: bool = ! matches.is_present("no_split_edge");
    let path: &str = match matches.value_of("FILE") {
        Some(x) => x,
        None => {
            log!(e:(1) "NoFile: append '-H' or '--help'");
        },
    };
    let mut colors: HashMap<String, String> = HashMap::new();
    {
        macro_rules! push {
            ( $x:expr ) => (
                macro_rules! err {
                    () => (log!(e:(2) "ColorsFormatError: {:?}", $x));
                }
                $x.split(',').map(|x| {
                    let mut kv = x.split(':');
                    let k: &str = if let Some(x) = kv.next() {x}
                        else {err!{}};
                    let v: &str = if let Some(x) = kv.next() {x}
                        else {err!{}};
                    if let Some(_) = kv.next() {err!{}}
                    colors.insert(k.to_string(), v.to_string());
                }).last();
            );
        }
        if ! matches.is_present("disable_default_colors") {
            push!(DEFAULT_COLORS);
        }
        match matches.value_of("colors") {
            Some(x) => {
                push!(x);
            },
            None => (),
        };
    }
    if matches.is_present("output_colors") {
        println!("{}", colors.keys().map(|x| format!("{}:{}", x, match colors.get(x) {
            Some(x) => x,
            None => {log!(e:(4) "MemError");},
        })).collect::<Vec<String>>().join(","));
        exit(0);
    }

    let opt_level: u8 = get_val!("opt_level", 1, parse);

    // background color
    let background_color: (u8, u8, u8)
        = match matches.value_of("background_color") {
            Some(x) => {
                if x.len() != 6 {
                    log!(e:(2) "HexColorLenError: len {} is not 6", x.len());
                }
                if let Some(n) = base16_to_unum(x) {
                    num_to_rgb(n)
                } else {
                    log!(e:(2) "HexColorFormatError: {}", x);
                }
            },
            _ => (0, 0, 0), // default value
        };

    let filter: FilterType
        = get_val!("filter",
            FilterType::Lanczos3,
            get_filter);


    let img = match read_img(path) {
        Ok(img) => {
            let (img_w, img_h): (u32, u32) 
                = (img.width(), img.height());
            (width, height)
                = if abs_size {
                    (width, height)
                } else {
                    image_size(
                        (img_w, img_h),
                        if has_set_size {
                            (width, height)
                        } else {
                            (term_width, term_height)
                    }, 0)
                };
            img.resize_exact(width, height, filter).into_rgba8()
        },
        Err(e) => {
            log!(e:(2) "{}", e);
        },
    };

    assert_eq!(width, img.width());
    assert_eq!(height, img.height());

    let width_usize: usize = width as usize;
    let mut line_num: u32 = 0;
    let mut bg_line_buffer: Vec<Rgb> = Vec::with_capacity(width_usize);
    let mut fg_line_buffer: Vec<Rgb> = Vec::with_capacity(width_usize);
    let mut mode: bool = false;
    let mut skip_fg_line: bool = false;
    let mut output_buffer: String = String::new();
    macro_rules! out {
        ( $( $x:expr ),* ) => (output_buffer.push_str(&format!( $($x),* )));
    }
    macro_rules! color_presets {
        ( $x:expr ) => {
            match colors.get(&$x) {
                Some(x) => x.clone(),
                None => $x,
        }};
    }
    macro_rules! color {
        (bf: $b:tt, $f:tt ) => {
            format!("{}[{};{}m", ESC, color_presets!(
                            format!("48;2;{};{};{}",
                                $b.0, $b.1, $b.2)),
                        color_presets!(
                            format!("38;2;{};{};{}",
                                $f.0, $f.1, $f.2)
                            )
        )};
        (b: $b:tt ) => (
            format!("{}[{}m", ESC, color_presets!(
                    format!("48;2;{};{};{}", $b.0, $b.1, $b.2))));
        (f: $f:tt ) => (
            format!("{}[{}m", ESC, color_presets!(
                    format!("38;2;{};{};{}", $f.0, $f.1, $f.2))));
    }
    let bg_ansi: String = color!(bf: background_color, background_color);
    let clear_ansi: String = format!("{}[{}m",
        ESC, color_presets!(String::from("0")));
    output_buffer.push_str(&bg_ansi);
    let (mut old_fg, mut old_bg): (Rgb, Rgb)
        = (Rgb::from(background_color),
            Rgb::from(background_color));
    'a: for color in img.pixels() {
        let mut tmp = Rgb::from(background_color);
        tmp.set_from_rgba((color.0[0], color.0[1],
                               color.0[2], color.0[3]));
        loop {
            match mode {
                false => {
                    bg_line_buffer.push(tmp);
                    if bg_line_buffer.len() == width_usize {
                        mode = true;
                        line_num += 1;
                        if line_num == height {
                            skip_fg_line = true;
                            continue;
                        }
                    }
                },
                true => { // output buffer
                    if skip_fg_line {
                        for _ in 0..width {
                            fg_line_buffer.push(Rgb::from(background_color));
                        }
                    } else {
                        fg_line_buffer.push(tmp);
                    }
                    if fg_line_buffer.len() == width_usize {
                        mode = false;
                        // output buffer color
                        if split_edge {
                            (old_fg, old_bg)
                                = (Rgb::from(background_color),
                                    Rgb::from(background_color));
                        }
                        let (mut fg, mut bg): (Rgb, Rgb);
                        let (mut fg_similar, mut bg_similar): (bool, bool); // 是否相似
                        let mut bg_fg_similar: bool;
                        let mut target_char: &str;
                        for i in 0..width_usize {
                            bg = bg_line_buffer[i];
                            fg = fg_line_buffer[i];
                            (fg_similar, bg_similar)
                                = (fg.is_similar(old_fg, opt_level),
                                bg.is_similar(old_bg, opt_level));
                            bg_fg_similar = enable_empty_char
                                && fg.is_similar(bg, opt_level);

                            target_char = if bg_fg_similar {
                                empty_char
                            } else {
                                fg_char
                            };

                            // 更新上一色的缓存
                            if ! fg_similar {
                                old_fg = fg;
                            }
                            if ! bg_similar {
                                old_bg = bg;
                            }

                            // to output_buffer
                            if bg_similar && fg_similar {
                                out!("{}", target_char);
                            } else if fg_similar {
                                out!("{}{}", color!(b: bg), target_char);
                            } else if bg_similar {
                                out!("{}{}", color!(f: fg), target_char);
                            } else {
                                out!("{}{}", color!(bf: bg, fg), target_char);
                            }
                        }
                        bg_line_buffer.clear();
                        fg_line_buffer.clear();
                        line_num += 1;
                        if line_num >= height {
                            break 'a;
                        }
                        if split_edge {
                            out!("{}{}{}", &clear_ansi, CR, &bg_ansi);
                        } else {
                            out!("{}", CR);
                        }
                    }
                },
            };
            break;
        }
    }
    println!("{}{}", output_buffer, &clear_ansi);
}
