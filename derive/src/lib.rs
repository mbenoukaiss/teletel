extern crate proc_macro;

use proc_macro::TokenStream;
use regex::Regex;
use std::str::FromStr;

/// Generates the code of a semi-graphic character. A semi-graphic character
/// is the size of a normal character but is divided into 2 columns and 3 rows.
/// Each of the 6 smaller squares can either display the foreground color or the
/// background color.
///
/// # Format
/// Input should be three slash-separated groups of 2 digits which are either
/// 1 or 0 where 1 means the square is filled and displays the foreground color,
/// or 0 which means the square is empty and displays the background color.
///
/// Or in a more concise way if you speak regex `^[01]{2}/[01]{2}/[01]{2}$`.
///
/// # Examples
/// In the following examples the documentation above the macro call shows the pattern
/// of the character that will be generated. An X means the square is filled (foreground
/// color is applied) and a - means there is nothing (background color is applied).
///
/// ```
/// /// XX
/// /// X-
/// /// --
/// let code = sq!(11/10/00);
///
/// /// --
/// /// --
/// /// --
/// let code = sq!(00/00/00);
///
/// /// XX
/// /// XX
/// /// XX
/// let code = sq!(11/11/11);
///
/// /// X-
/// /// X-
/// /// X-
/// let code = sq!(10/10/10);
/// ```
#[proc_macro]
pub fn sg(input: TokenStream) -> TokenStream {
    let format = Regex::new(r"^[01]{2}/[01]{2}/[01]{2}$").unwrap();
    let input = input.to_string();

    if !format.is_match(&input) {
        panic!("Invalid semi-graphic character format, expected `[01]{{2}}/[01]{{2}}/[01]{{2}}` got `{}`", input)
    }

    let mut binary = input
        .chars()
        .filter(|c| c.is_digit(2))
        .map(|c| c.to_digit(2).unwrap())
        .rev()
        .fold(0, |acc, digit| (acc << 1) + digit);

    binary += 0x20;
    if binary >= 0x40 {
        binary += 0x20;
    }

    if binary == 0x7F {
        binary = 0x5F;
    }

    TokenStream::from_str(&binary.to_string()).unwrap()
}
