use std::str::FromStr;

use toiletcli::colors::*;

fn main() -> () {
    println!("Byte of (255, 0, 0) is {:?} (should be 196)!", Color::RGB(255, 0, 0).byte());
    assert_eq!(Color::RGB(255, 0, 0).byte(), 196);

    println!("{}owo owo owo owo owo owo owo{}", Color::RGB(255, 200, 200), Style::Reset);

    println!("Color from 'red' is {:?}", Color::from_str("red"));
    assert_eq!(Color::from_str("red").unwrap(), Color::Red);

    println!("Color from 'bright-white' is {:?}", Color::from_str("bright-white"));
    assert_eq!(Color::from_str("bright-white").unwrap(), Color::BrightWhite);
    println!("{}Bright white background!{}", Color::BrightWhite.bg(), Style::Reset);

    println!("{}Red text!{}", Color::Red, Style::Reset);

    println!(
        "{}{}Blue on bright red text!{}",
        Color::Blue,
        Color::BrightRed.bg(),
        Style::Reset
    );

    println!(
        "{}{}{}Italic bold cyan text!{}",
        Style::Italic,
        Style::Bold,
        Color::Cyan,
        Style::Reset
    );

    let my_style = StyleBuilder::new()
        .background(Color::BrightGreen)
        .add_style(Style::Italic)
        .add_style(Style::Strikethrough)
        .build();

    println!(
        "{}This is striked and italic text on bright green background!{}",
        my_style,
        Style::Reset
    );

    let warning = StyleBuilder::new()
        .foreground(Color::BrightBlack)
        .background(Color::Red)
        .add_style(Style::Bold)
        .build();

    let warning_message = StyleBuilder::new()
        .foreground(Color::Purple)
        .add_style(Style::Underlined)
        .underline_color(Color::Green)
        .underline_style(UnderlineStyle::Curly)
        .build();

    println!(
        "{}Bold bright black on red:{} {}Purple with curly green underline!{}",
        warning,
        Style::Reset,
        warning_message,
        Style::Reset
    );

    println!("{}{}This is red text on blue background!{}",
         Color::Red, Color::Blue.bg(), Style::Reset);

    let weird_style = StyleBuilder::new()
        .foreground(Color::Byte(93))
        .add_style(Style::Underlined)
        .underline_color(Color::RGB(0, 255, 0))
        .underline_style(UnderlineStyle::Curly)
        .build();

    println!("{}RGB purple with RGB curly green underline!{}",
            weird_style, Style::Reset);

    println!("{}{}Pink 8-bit color!{}", Color::Black, Color::Byte(218).bg(), Style::Reset);

    println!("{}RGB red color{}", Color::RGB(255, 0, 0), Style::Reset);

    let rgb_underline = StyleBuilder::new()
        .foreground(Color::RGB(255, 255, 0))
        .add_style(Style::Underlined)
        .underline_color(Color::RGB(0, 0, 255))
        .build();

    println!("{}Yellow RGB color with very blue RGB underline!{}", rgb_underline, Style::Reset);
}
