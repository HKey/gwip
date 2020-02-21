use std::process;
use std::process::Command;
use docopt::Docopt;
use serde::Deserialize;
use regex::Regex;


struct WindowGeometry {
    // x: u32,
    // y: u32,
    w: u32,
    h: u32,
}

struct DisplayGeometry {
    w: u32,
    h: u32,
}

type WindowID = u32;

// FIXME: unwrapping in utility functions may be not good
fn get_window_geometry(id: WindowID, xwininfo: &str) -> WindowGeometry {
    let output = Command::new(xwininfo)
        .arg("-id")
        .arg(id.to_string())
        .output()
        .expect(&format!("Failed to execute {}", xwininfo))
        .stdout;
    let output = String::from_utf8(output).unwrap();

    fn parse_value(header: &str, output: &str) -> u32 {
        let re = format!(r"(?m)^[ ]*{}[ ]+(\d+)$", regex::escape(&header));
        Regex::new(&re).unwrap()
            .captures(&output).unwrap()[1]
            .parse().unwrap()
    }

    WindowGeometry {
        // x: parse_value("Absolute upper-left X:", &output),
        // y: parse_value("Absolute upper-left Y:", &output),
        w: parse_value("Width:", &output),
        h: parse_value("Height:", &output),
    }
}

fn get_focused_window_id(xdotool: &str) -> WindowID {
    let output = Command::new(xdotool)
        .arg("getactivewindow")
        .output()
        .expect(&format!("Failed to execute {}", xdotool))
        .stdout;
    String::from_utf8(output).unwrap()
        .trim()
        .parse().unwrap()
}

fn get_display_geometry(xdotool: &str) -> DisplayGeometry {
    let output = Command::new(xdotool)
        .arg("getdisplaygeometry")
        .output()
        .expect(&format!("Failed to execute {}", xdotool))
        .stdout;
    let output = String::from_utf8(output).unwrap();
    let output = output.trim();
    let result: Vec<&str> = output.split(" ").collect();

    DisplayGeometry {
        w: result[0].parse().unwrap(),
        h: result[1].parse().unwrap(),
    }
}

fn move_window(id: WindowID, x: u32, y: u32, xdotool: &str) {
    Command::new(xdotool)
        .arg("windowmove")
        .arg(id.to_string())
        .arg(x.to_string())
        .arg(y.to_string())
        .status()
        .expect(&format!("Failed to execute {}", xdotool));
}

fn move_window_in_grid(id: WindowID,
                       nlines: u32, ncolumns: u32,
                       line: u32, column: u32,
                       xdotool: &str, xwininfo: &str) {
    let display = get_display_geometry(xdotool);
    let window = get_window_geometry(id, xwininfo);

    assert!(nlines > 0);
    assert!(ncolumns > 0);
    assert!(line < nlines);
    assert!(column < ncolumns);

    let cell_w = (display.w / ncolumns) as i64;
    let cell_h = (display.h / nlines) as i64;
    let line = line as i64;
    let column = column as i64;
    let win_w = window.w as i64;
    let win_h = window.h as i64;

    let new_x = cell_w * column + (cell_w - win_w) / 2;
    let new_y = cell_h * line + (cell_h - win_h) / 2;

    fn i64_to_u32_minus_as_zero(n: i64) -> Option<u32> {
        if n > (u32::max_value() as i64) {
            return None
        } else if n < (u32::min_value() as i64) {
            return Some(0)
        } else {
            return Some(n as u32)
        }
    }

    move_window(id,
                i64_to_u32_minus_as_zero(new_x).unwrap(),
                i64_to_u32_minus_as_zero(new_y).unwrap(),
                xdotool);
}


const USAGE: &'static str = "
Gridded Window Placer

Usage:
    gwip move --grid=<NLINESxNCOLUMNS> --place=<LINE,COLUMN>
              [--xdotool=<cmd>] [--xwininfo=<cmd>]
    gwip -h | --help

Options:
    -h, --help                Show this screen.
    --grid=<NLINESxNCOLUMNS>  How to divide screen.
                              Example: \"--grid=2x1\"
    --place=<LINE,COLUMN>     Where move the focused window to.
                              Example: \"--place=0,0\"
    --xdotool=<cmd>           Command of xdotool. [default: xdotool]
    --xwininfo=<cmd>          Command of xwininfo. [default: xwininfo]

Commands:
    move
        Move the focused window to the specified place.
        The display is divided into grid by \"--grid\" parameter.
        The window will be moved to the center of the cell in the grid
        specified by \"--place\" parameter.
        The line and the column of a cell is counted from top left
        and zero based.

        Example:
            $ gwip move --grid=2x3 --place=0,1

                            ncolumns = 3
                      |-----------------------|

                   -  +-------+-------+-------+
                   |  |  0,0  |  0,1  |  0,2  |
                   |  |       | here! |       |
        nlines = 2 |  |-------+-------+-------|
                   |  |  1,0  |  1,1  |  1,2  |
                   |  |       |       |       |
                   -  +-------+-------+-------+
";

#[derive(Debug, Deserialize)]
struct Args {
    flag_grid: String,
    flag_place: String,
    flag_xdotool: String,
    flag_xwininfo: String,
    cmd_move: bool,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    // command missing
    if !args.cmd_move {
        eprintln!("Invalid arguments, see \"gwip --help\"");
        process::exit(1);
    }

    // move command
    if args.cmd_move {
        fn parse_grid(s: &str) -> u32 {
            let msg = &format!("Invalid argument, \"{}\" in --grid", s);
            s.parse().expect(msg)
        }

        let grid: Vec<&str> = args.flag_grid.split("x").collect();
        assert!(grid.len() == 2);
        let nlines: u32 = parse_grid(grid[0]);
        let ncolumns: u32 = parse_grid(grid[1]);


        fn parse_place(s: &str) -> u32 {
            let msg = &format!("Invalid argument, \"{}\" in --place", s);
            s.parse().expect(msg)
        }

        let place: Vec<&str> = args.flag_place.split(",").collect();
        assert!(place.len() == 2);
        let line: u32 = parse_place(place[0]);
        let column: u32 = parse_place(place[1]);

        let id = get_focused_window_id(&args.flag_xdotool);

        move_window_in_grid(id, nlines, ncolumns, line, column,
                            &args.flag_xdotool, &args.flag_xwininfo);
    }
}
