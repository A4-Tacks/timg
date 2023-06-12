pub type SizeType = u32;
pub type Float = f64;


mod traits {
    use term_lattice::Color;

    pub trait FmtColor {
        fn fmt_color(&self) -> String;
    }
    impl FmtColor for Color {
        /// format color
        /// # Examples
        /// ```
        /// # use term_lattice::Color;
        /// # use timg::FmtColor;
        /// assert_eq!(Color::None.fmt_color(), "None");
        /// assert_eq!(Color::Rgb([0xff, 0x1b, 0x0a]).fmt_color(),
        ///     "\x1b[48;2;255;27;10m#\x1b[49mFF1B0A");
        /// assert_eq!(Color::C256(84).fmt_color(), "\x1b[48;5;84mC\x1b[49m084");
        /// ```
        fn fmt_color(&self) -> String {
            match self {
                Self::None => "None".to_string(),
                Self::Rgb(x)
                    => format!("\x1b[48;2;{0};{1};{2}m#\x1b[49m{:02X}{:02X}{:02X}",
                               x[0], x[1], x[2]),
                Self::C256(x) => format!("\x1b[48;5;{0}mC\x1b[49m{:03}", x),
            }
        }
    }

    pub trait IsAlpha {
        fn is_alpha(&self) -> bool;
    }
}
pub use traits::*;


/// as float
#[macro_export]
macro_rules! asf {
    ( $($x:expr),* ) => {
        ( $($x as $crate::Float),* )
    };
}


mod position {
    use std::ops;

    use super::*;

    /// 一个位置 (x, y)
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Position {
        pub x: SizeType,
        pub y: SizeType,
    }
    impl Position {
        pub fn new(x: SizeType, y: SizeType) -> Self {
            Self { x, y }
        }
        pub fn into_array(self) -> [SizeType; 2] {
            [self.x, self.y]
        }
        /// 这不会改变原数据, 而是从栈上 copy 一份
        /// # Examples
        /// ```
        /// use timg::Position;
        /// let a = Position::new(2, 3);
        /// let b = Position::new(6, 9);
        /// assert_eq!(a.mul_scale(3.0), b);
        /// assert_eq!(a.mul_scale(3.0), b);
        /// ```
        pub fn mul_scale(mut self, num: Float) -> Self {
            let (mut x, mut y) = asf!(self.x, self.y);
            x *= num;
            y *= num;
            self.x = x as SizeType;
            self.y = y as SizeType;
            self
        }
    }
    impl From<[SizeType; 2]> for Position {
        fn from(value: [SizeType; 2]) -> Self {
            Self::new(value[0], value[1])
        }
    }
    impl From<(SizeType, SizeType)> for Position {
        fn from(value: (SizeType, SizeType)) -> Self {
            Self::new(value.0, value.1)
        }
    }
    impl From<SizeType> for Position {
        fn from(value: SizeType) -> Self {
            Self::new(value, value)
        }
    }
    impl Default for Position {
        /// (0, 0)
        fn default() -> Self {
            Self::new(0, 0)
        }
    }
    macro_rules! impl_ops {
        (($oper:tt) =>
         [$tname:path, $fname:ident],
         [$tname1:path, $fname1:ident]) => {
            impl $tname for Position {
                type Output = Self;
                fn $fname(mut self, rhs: Self) -> Self::Output {
                    self.x $oper rhs.x;
                    self.y $oper rhs.y;
                    self
                }
            }
            impl $tname1 for Position {
                fn $fname1(&mut self, rhs: Self) {
                    self.x $oper rhs.x;
                    self.y $oper rhs.y;
                }
            }
        };
    }
    impl_ops!((+=) => [ops::Add, add], [ops::AddAssign, add_assign]);
    impl_ops!((-=) => [ops::Sub, sub], [ops::SubAssign, sub_assign]);
    impl_ops!((*=) => [ops::Mul, mul], [ops::MulAssign, mul_assign]);
    impl_ops!((/=) => [ops::Div, div], [ops::DivAssign, div_assign]);
    impl_ops!((%=) => [ops::Rem, rem], [ops::RemAssign, rem_assign]);
    impl_ops!((>>=) => [ops::Shr, shr], [ops::ShrAssign, shr_assign]);
    impl_ops!((<<=) => [ops::Shl, shl], [ops::ShlAssign, shl_assign]);
    impl_ops!((&=) => [ops::BitAnd, bitand], [ops::BitAndAssign, bitand_assign]);
    impl_ops!((|=) => [ops::BitOr, bitor], [ops::BitOrAssign, bitor_assign]);
    impl_ops!((^=) => [ops::BitXor, bitxor], [ops::BitXorAssign, bitxor_assign]);

    #[test]
    fn test() {
        let mut a = Position::new(3, 8);
        assert_eq!(a + Position::new(4, 1), Position::new(7, 9));
        a += Position::new(4, 1);
        assert_eq!(a, Position::new(7, 9));
    }
}
pub use position::*;

/// 获取要将图片大小缩到刚好放进终端大小时, 终端大小须乘的比例
/// 终端大小 * 比例 得到刚好包括整个图片的大小
pub fn get_scale(term_size: Position, img_size: Position) -> Float {
    let (tw, th, iw, ih)
        = asf!(term_size.x, term_size.y, img_size.x, img_size.y);
    if tw / iw < th / ih {
        /* 当 y 轴合适时 img.x 过长 */
        iw / tw
    } else {
        /* 当 x 轴合适时 img.y 过长 */
        ih / th
    }
}
#[test]
fn get_scale_test() {
    let term_size = Position::new(80, 60);
    let img_size = Position::new(240, 740);
    let scale = get_scale(term_size, img_size);
    assert!(term_size.x as Float * scale >= img_size.x as Float);
    assert_eq!(term_size.y as Float * scale, img_size.y as Float);

    let term_size = Position::new(80, 60);
    let img_size = Position::new(240, 40);
    let scale = get_scale(term_size, img_size);
    assert!(term_size.y as Float * scale >= img_size.y as Float);
    assert_eq!(term_size.x as Float * scale, img_size.x as Float);
}


pub const ESC: char = '\x1b';
pub const CR: &str = "\n";


/// # Examples
/// ```
/// use timg::default_value;
/// let a = 0.0;
/// let result = 5.0 / default_value!(a => 0.0, 1.0);
/// assert_eq!(result, 5_f32);
/// ```
#[macro_export]
macro_rules! default_value {
    ( $x:expr => $v:expr , $default:expr )
        => (if $x != $v { $x } else { $default });
}

/// # Examples
/// ```
/// use timg::pass;
/// assert_eq!(pass!(((8, 9, 12)) !(13, 15)), (8, 9, 12));
/// assert_eq!(pass!((6) !(13, 15)), 6);
/// ```
#[macro_export]
macro_rules! pass {
    ( ($( $v:tt )*) $( !( $( $d:expr ),* ) )? ) => {
        $( $v )*
    };
}

/// # Examples
/// ```
/// use timg::join_str;
/// use timg::pass;
/// let a = 2;
/// let b = "hello";
/// assert_eq!(join_str!("hi,", a, b), "hi,2hello");
/// ```
#[macro_export]
macro_rules! join_str {
    ( $( $x:expr ),+ ) => (format!(concat!($(pass!(("{}") !($x))),+), $($x),+));
}


/// # Examples
/// ```
/// use timg::base16_to_unum;
/// let n = base16_to_unum("abfF14");
/// assert_eq!(n, Some(0xabff14));
/// ```
pub fn base16_to_unum(s: &str) -> Option<u32> {
    let mut num: u32 = 0;
    for i in s.chars() {
        num <<= 4;
        num |= match i {
            '0' => 0, '1' => 1, '2' => 2, '3' => 3, '4' => 4,
            '5' => 5, '6' => 6, '7' => 7, '8' => 8, '9' => 9,
            'a'|'A' => 10, 'b'|'B' => 11, 'c'|'C' => 12,
            'd'|'D' => 13, 'e'|'E' => 14, 'f'|'F' => 15,
            _ => {
                return None;
            },
        };
    }
    Some(num)
}

/// # Examples
/// ```
/// use timg::num_to_rgb;
/// assert_eq!(num_to_rgb(0xff0084), [0xff, 0x00, 0x84]);
/// ```
pub fn num_to_rgb(num: u32) -> [u8; 3] {
    // base16: 4bit
    // base16 * 2: 8bit
    debug_assert!(num <= 0xffffff);

    [((num >> 16) & 0xff) as u8,
     ((num >> 8) & 0xff) as u8,
     (num & 0xff) as u8]
}
