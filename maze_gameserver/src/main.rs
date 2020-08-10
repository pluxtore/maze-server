#![allow(dead_code)]
#![allow(non_upper_case_globals)]


mod gamelogic;
mod aluminium;

use std::*;
use fern::colors::{Color, ColoredLevelConfig};


// ===================== Logging Set Up =====================
fn set_up_logging() -> Result<(), fern::InitError> {
    // configure colors for the whole line
    let colors_line = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        // we actually don't need to specify the color for debug and info, they are white by default
        .info(Color::White)
        .debug(Color::Cyan)
        // depending on the terminals color scheme, this is the same as the background color
        .trace(Color::BrightBlack);

    // configure colors for the name of the level.
    // since almost all of them are the same as the color for the whole line, we
    // just clone `colors_line` and overwrite our changes
    let colors_level = colors_line.clone().info(Color::Green);
    // here we set up our fern Dispatch
    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{color_line}[{date}][{target}][{level}{color_line}] {message}\x1B[0m",
                color_line = format_args!(
                    "\x1B[{}m",
                    colors_line.get_color(&record.level()).to_fg_str()
                ),
                date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                target = record.target(),
                level = colors_level.color(record.level()),
                message = message,
            ));
        })
        // set the default log level. to filter out verbose log messages from dependencies, set
        // this to Warn and overwrite the log level for your crate.
        .level(log::LevelFilter::Trace)
        // change log levels for individual modules. Note: This looks for the record's target
        // field which defaults to the module path but can be overwritten with the `target`
        // parameter:
        // `info!(target="special_target", "This log message is about special_target");`
        .level_for("pretty_colored", log::LevelFilter::Trace)
        // output to stdout
        .chain(std::io::stdout())
        .chain(fern::log_file("aluminium.log")?)
        .apply()
        .unwrap();

    Ok(())
}

fn _main() {
    match set_up_logging() {
        Ok(_e) => (),
        Err(_e) => println!("failed to initialize logger"),
    }
    let word = b"hello test test test abc lul f f f f fnsdfasdfjoisdfgjdf   dsfhjadfhgosdfgojd  fjggdfh adfggodnfgdfigdfgnadongojfdnb";

    if word.to_vec() == gamelogic::Logic::decode_pkt(gamelogic::Logic::encode_pkt(word.to_vec()).as_mut_slice()) {
        log::info!("decode / encode test passed successfully");
    } else {
        log::error!("decode / encode test failed");
    }


    if (fs::metadata("clients/")).is_err() {
        match fs::create_dir("clients") {
            Ok(_e) => log::info!("created directory clients/ "),
            Err(_e) => log::error!(
                "lacking permissions to create clients directory, this will lead to future errors"
            ),
        };
    }

    log::info!("starting game server...");
    aluminium::Server::start((1337, 1339));
}

fn main() {
    _main();
    std::thread::park();
}
