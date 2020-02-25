use std::process;
use std::process::Command;
use std::cmp;
use docopt::Docopt;
use serde::Deserialize;
use regex::Regex;


struct WindowGeometry {
    // x: i32,
    // y: i32,
    w: i32,
    h: i32,
}

struct DisplayGeometry {
    w: i32,
    h: i32,
}

type WindowID = i32;

// FIXME: unwrapping in utility functions may be not good
fn get_window_geometry(id: WindowID, xwininfo: &str) -> WindowGeometry {
    let output = Command::new(xwininfo)
        .arg("-id")
        .arg(id.to_string())
        .output()
        .expect(&format!("Failed to execute {}", xwininfo))
        .stdout;
    let output = String::from_utf8(output).unwrap();

    fn parse_value(header: &str, output: &str) -> i32 {
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

fn move_window(id: WindowID, x: i32, y: i32, xdotool: &str) {
    Command::new(xdotool)
        .arg("windowmove")
        .arg(id.to_string())
        .arg(x.to_string())
        .arg(y.to_string())
        .status()
        .expect(&format!("Failed to execute {}", xdotool));
}

fn move_window_in_grid(id: WindowID,
                       nlines: i32, ncolumns: i32,
                       line: i32, column: i32,
                       xdotool: &str, xwininfo: &str) {
    let display = get_display_geometry(xdotool);
    let window = get_window_geometry(id, xwininfo);

    assert!(nlines > 0);
    assert!(ncolumns > 0);
    assert!(line < nlines);
    assert!(column < ncolumns);

    let cell_w = display.w / ncolumns;
    let cell_h = display.h / nlines;

    let x = cell_w * column + (cell_w - window.w) / 2;
    let y = cell_h * line + (cell_h - window.h) / 2;

    let x = cmp::max(cmp::min(x, display.w - window.w), 0);
    let y = cmp::max(cmp::min(y, display.h - window.h), 0);

    move_window(id, x, y, xdotool);
}

fn resize_window(id: WindowID, width: i32, height: i32, xdotool: &str)
                 -> Result<(), std::io::Error> {
    Command::new(xdotool)
        .arg("windowsize")
        .arg(id.to_string())
        .arg(width.to_string())
        .arg(height.to_string())
        .status()
        .map(|_| ())
}

fn resize_window_to_fill(id: WindowID, nlines: i32, ncolumns: i32,
                         gap_w: i32, gap_h: i32,
                         display: &DisplayGeometry, xdotool: &str)
                         -> Result<(), std::io::Error> {
    let width = display.w / ncolumns - gap_w * 2;
    let height = display.h / nlines - gap_h * 2;

    resize_window(id, width, height, xdotool)
}


const USAGE: &'static str = "
Gridded Window Placer

Usage:
    gwip move --grid=<NLINESxNCOLUMNS> --place=<LINE,COLUMN> [options]
    gwip -h | --help

Options:
    -h, --help                Show this screen.
    --grid=<NLINESxNCOLUMNS>  How to divide screen.
                              Example: \"--grid=2x1\"
    --place=<LINE,COLUMN>     Where move the focused window to.
                              Example: \"--place=0,0\"
    --fill                    Resize the window to fill the cell where the
                              the window will be moved to.
    --gap=<WIDTHxHEIGHT>      For \"--fill\", do not fill each edge of the
                              window.  WIDTH is a width of left and right edge.
                              HEIGHT is a height of top and bottom edge.
                              WIDTH and HEIGHT is a number or a percentage.
                              A number means the number of pixels, a percentage
                              means the percentage of the desktop width
                              or height.
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
    flag_fill: bool,
    flag_gap: Option<String>,
    flag_xdotool: String,
    flag_xwininfo: String,
    cmd_move: bool,
}

enum IntOrPercent {
    Int(i32),
    Percent(f32),
}

fn parse_int_or_percent(s: &str) -> Result<IntOrPercent, String> {
    let s = s.trim();
    if s.ends_with("%") {
        s[..s.len()-1]
            .parse()
            .map(|p: f32| IntOrPercent::Percent(p / 100.0))
            .map_err(|e: std::num::ParseFloatError| e.to_string())
    } else {
        s.parse()
            .map(|i| IntOrPercent::Int(i))
            .map_err(|e| e.to_string())
    }
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
        fn parse_grid(s: &str) -> i32 {
            let msg = &format!("Invalid argument, \"{}\" in --grid", s);
            s.parse().expect(msg)
        }

        let grid: Vec<&str> = args.flag_grid.split("x").collect();
        assert!(grid.len() == 2);
        let nlines: i32 = parse_grid(grid[0]);
        let ncolumns: i32 = parse_grid(grid[1]);


        fn parse_place(s: &str) -> i32 {
            let msg = &format!("Invalid argument, \"{}\" in --place", s);
            s.parse().expect(msg)
        }

        let place: Vec<&str> = args.flag_place.split(",").collect();
        assert!(place.len() == 2);
        let line: i32 = parse_place(place[0]);
        let column: i32 = parse_place(place[1]);

        let id = get_focused_window_id(&args.flag_xdotool);

        if args.flag_fill {
            let display = get_display_geometry(&args.flag_xdotool);

            let mut gap_w = 0;
            let mut gap_h = 0;

            if let Some(s) = &args.flag_gap {
                fn percent_to_pixel(ip: IntOrPercent, base: i32) -> i32 {
                    match ip {
                        IntOrPercent::Int(i) => i,
                        IntOrPercent::Percent(f) => (f * base as f32) as i32,
                    }
                }

                let gaps: Vec<&str> = s.split("x").collect();
                gap_w = percent_to_pixel(parse_int_or_percent(&gaps[0])
                                         .unwrap(),
                                         display.w);
                gap_h = percent_to_pixel(parse_int_or_percent(&gaps[1])
                                         .unwrap(),
                                         display.w);
            }

            resize_window_to_fill(id, nlines, ncolumns,
                                  gap_w, gap_h,
                                  &display, &args.flag_xdotool).unwrap();
        }
        move_window_in_grid(id, nlines, ncolumns, line, column,
                            &args.flag_xdotool, &args.flag_xwininfo);
    }
}
