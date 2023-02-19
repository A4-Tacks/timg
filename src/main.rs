use clap::{
    Arg,
    App,
    ArgMatches,
};

mod app;

fn main() {
    let matches: ArgMatches
        = App::new(env!("CARGO_PKG_NAME"))

        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))

        .arg(Arg::with_name("width")
             .short("w")
             .long("width")
             .value_name("width")
             .help("Set display image width")
             .takes_value(true))

        .arg(Arg::with_name("height")
             .short("h")
             .long("height")
             .value_name("height")
             .help("Set display image height")
             .takes_value(true))

        .arg(Arg::with_name("foreground")
             .short("f")
             .long("foreground")
             .value_name("char")
             .help("Set display foreground char")
             .takes_value(true))

        .arg(Arg::with_name("opt_level")
             .short("d")
             .long("optimization-level")
             .value_name("level")
             .help(concat!(
                 "Set optimization level\n",
                 "(default:1) [0, 255]"))
             .takes_value(true))

        .arg(Arg::with_name("background_color")
             .short("b")
             .long("background-color")
             .value_name("color")
             .help(concat!(
                "Display image background color\n",
                "(default:000000)"))
             .takes_value(true))

        .arg(Arg::with_name("no_split_edge")
             .short("s")
             .long("no-split-edge")
             .help("No split edge"))

        .arg(Arg::with_name("filter")
             .short("t")
             .long("filter")
             .value_name("id")
             .help(concat!(
                 "Type of filter used\n",
                 "0: Nearest (quick)\n",
                 "1: Triangle\n",
                 "2: CatmullRom\n",
                 "3: Gaussian\n",
                 "4: Lanczos3 (default)",
             ))
             .takes_value(true))

        .arg(Arg::with_name("colors")
             .long("colors")
             .value_name("colors")
             .help(concat!(
                     "Define color output.\n",
                     "format: 38;2;0;0;0:30,48;2;0;0;0:40"
                     ))
             .takes_value(true))

        .arg(Arg::with_name("output_colors")
             .long("output-colors"))

        .arg(Arg::with_name("disable_default_colors")
             .long("disable-default-colors"))

        .args(&[
            Arg::with_name("FILE").index(1)
                .help("Target file")
        ])

        .help_short("H") // help flag
        .get_matches();

    app::run(matches);
}
