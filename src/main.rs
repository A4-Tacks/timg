use clap::{
    Arg,
    App,
    ArgMatches,
};

mod app;

macro_rules! lines {
    ( $first:tt $( $x:tt )* ) => {
        concat!(
            $first
            $(
                , "\n",
                $x
            )*
        )
    };
}

fn main() {
    let matches: ArgMatches
        = App::new(env!("CARGO_PKG_NAME"))

        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))

        .arg(Arg::with_name("term_size")
             .short("t")
             .long("term-size")
             .value_name("size")
             .takes_value(true)
             .help(lines!(
                     "When this value is set, it will be used as the terminal size."
                     "Format: (columns,lines)"
                     "Example: 80,40")))

        .arg(Arg::with_name("bgs")
             .short("b")
             .long("background-colors")
             .value_name("colors")
             .takes_value(true)
             .help(lines!(
                     "Set background colors."
                     "Default: 000000,888888,ffffff")))

        .arg(Arg::with_name("zoom_ratio")
             .short("z")
             .long("zoom-ratio")
             .value_name("num")
             .takes_value(true)
             .help(lines!(
                     "Number multiplied during scaling"
                     "Range: 0 < num < 1"
                     "Default: 0.8")))

        .arg(Arg::with_name("short_move_ratio")
             .short("s")
             .long("short-move-ratio")
             .value_name("num")
             .takes_value(true)
             .help(lines!(
                     "short-move The distance to move at once is num screen sizes"
                     "Range: num > 0"
                     "Default: 0.25")))

        .arg(Arg::with_name("opt_level")
             .short("o")
             .long("opt-level")
             .value_name("level")
             .takes_value(true)
             .help(lines!(
                     "Default optimization level"
                     "Range: num >= 0"
                     "Default: 60")))

        .arg(Arg::with_name("long_move_ratio")
             .short("l")
             .long("long-move-ratio")
             .value_name("num")
             .takes_value(true)
             .help(lines!(
                     "long-move The distance to move at once is num screen sizes"
                     "Range: num > 0"
                     "Default: 0.75")))

        .args(&[
            Arg::with_name("FILE").index(1)
                .help("Target file")
        ])

        .help_short("H") // help flag
        .get_matches();

    app::run(matches);
}
