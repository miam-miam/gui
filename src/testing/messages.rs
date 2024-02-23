use std::io;
use std::io::Write;
use std::path::Path;
use termcolor::{Color, ColorChoice, ColorSpec, HyperlinkSpec, StandardStream, WriteColor};

pub(crate) fn start_test(filename: &str, message: &str) -> io::Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    write!(&mut stdout, "Testing {filename} ")?;
    stdout.set_color(ColorSpec::new().set_bold(true))?;
    write!(&mut stdout, "{message}")?;
    stdout.reset()?;
    write!(&mut stdout, " ... ")?;
    Ok(())
}

pub(crate) fn print_pass_test() -> io::Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
    writeln!(&mut stdout, "nice!")?;
    stdout.reset()?;
    Ok(())
}

pub(crate) fn print_fail_test(expected: &Path, found: &Path) -> io::Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);

    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
    writeln!(&mut stdout, "error")?;

    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)).set_bold(true))?;
    writeln!(&mut stdout, "EXPECTED:")?;
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)))?;
    write!(&mut stdout, "file://{}", expected.display())?;
    stdout.reset()?;
    writeln!(&mut stdout)?;

    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))?;
    writeln!(&mut stdout, "FOUND:")?;
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
    write!(&mut stdout, "file://{}", found.display())?;
    stdout.reset()?;
    writeln!(&mut stdout)?;
    Ok(())
}

pub(crate) fn print_wip_test(expected: &Path, wip: &Path) -> io::Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);

    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))?;
    writeln!(&mut stdout, "wip")?;

    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)).set_bold(true))?;
    writeln!(&mut stdout, "NOTE:")?;
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))?;
    write!(&mut stdout, "Writing new image to ")?;
    write!(&mut stdout, "file://{}", wip.display())?;
    writeln!(&mut stdout, ".")?;
    writeln!(
        &mut stdout,
        "Rename the file to `{}` to accept it as correct.",
        expected.display()
    )?;
    stdout.reset()?;
    Ok(())
}
