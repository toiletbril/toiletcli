use std::str::FromStr;

use toiletcli::colors::*;

fn main() -> () {
    println!("Byte of (255, 0, 0) is {:?} (should be 196)!", Color::RGB(255, 0, 0).byte());

    println!("Color from 'red' is {:?}", Color::from_str("red"));

    println!("{}Red text!{}", Color::Red, Style::Reset);

    println!(
        "{}{}Blue on bright red text!{}",
        Color::Blue,
        Color::BrightRed.background(),
        Style::Reset
    );

    println!(
        "{}{}{}Italic bold cyan text!{}",
        Style::Italic,
        Style::Bold,
        Color::Cyan,
        Style::Reset
    );

    let mut my_style = TerminalStyle::new();
    my_style
        .background(Color::BrightGreen)
        .add_style(Style::Italic)
        .add_style(Style::Strikethrough);

    println!(
        "{}This is striked and italic text on bright green background!{}",
        my_style,
        Style::Reset
    );

    let mut warning = TerminalStyle::new();
    warning
        .foreground(Color::BrightBlack)
        .background(Color::Red)
        .add_style(Style::Bold);

    let mut warning_message = TerminalStyle::new();
    warning_message
        .foreground(Color::Purple)
        .add_style(Style::Underlined)
        .underline_color(Color::Green)
        .underline_style(UnderlineStyle::Curly);

    println!(
        "{}Bold bright black on red:{} {}Purple with curly green underline!{}",
        warning,
        Style::Reset,
        warning_message,
        Style::Reset
    );

    println!("{}{}This is red text on blue background!{}",
         Color::Red, Color::Blue.background(), Style::Reset);

    let mut weird = TerminalStyle::new();
    weird
        .foreground(Color::Byte(93))
        .add_style(Style::Underlined)
        .underline_color(Color::RGB(0, 255, 0))
        .underline_style(UnderlineStyle::Curly);

    println!("{}Purple with curly green underline!{}",
            warning_message, Style::Reset);

    println!("{}{}Pink 256 color!{}", Color::Black, Color::Byte(218).background(), Style::Reset);

    println!("{}True red color{}", Color::RGB(255, 0, 0), Style::Reset);

    let mut rgb_underline = TerminalStyle::new();
    rgb_underline
        .foreground(Color::RGB(255, 0, 0))
        .add_style(Style::Underlined)
        .underline_color(Color::RGB(0, 0, 255));

    println!("{}Very red RGB color with very blue RGB underline!{}", rgb_underline, Style::Reset);
}
