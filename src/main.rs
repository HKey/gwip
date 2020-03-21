use std::process;
use std::process::{Command, Output};
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

struct WindowDecoration {
    l: i32,
    r: i32,
    t: i32,
    b: i32,
}

type WindowID = i32;

// Run an external command.
// If the command returns non zero exit code, this function panics.
fn run_with_check(command: &[&str]) -> Output {
    if command.len() < 1 {
        panic!("Program name is missing");
    }
    let program = &command[0];
    let args = &command[1..];
    let result = Command::new(program)
        .args(args)
        .output();

    match result {
        Err(e) => panic!("{}", e),
        Ok(output) => if output.status.success() {
            output
        } else {
            panic!("Command faild: {}\n\
                    --- status code ---\n\
                    {}\n\
                    --- stdout ---\n\
                    {}\n\
                    --- stderr ---\n\
                    {}",
                   command.join(" "),
                   match output.status.code() {
                       Some(n) => format!("{}", n),
                       None => String::from("none")
                   },
                   String::from_utf8(output.stdout).unwrap(),
                   String::from_utf8(output.stderr).unwrap());
        }
    }
}

// FIXME: unwrapping in utility functions may be not good
fn get_window_geometry(id: WindowID, xwininfo: &str) -> WindowGeometry {
    let output = run_with_check(&[xwininfo, "-id", &id.to_string()]).stdout;
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
    let output = run_with_check(&[xdotool, "getactivewindow"]).stdout;
    String::from_utf8(output).unwrap()
        .trim()
        .parse().unwrap()
}

fn get_display_geometry(xdotool: &str) -> DisplayGeometry {
    let output = run_with_check(&[xdotool, "getdisplaygeometry"]).stdout;
    let output = String::from_utf8(output).unwrap();
    let output = output.trim();
    let result: Vec<&str> = output.split(" ").collect();

    DisplayGeometry {
        w: result[0].parse().unwrap(),
        h: result[1].parse().unwrap(),
    }
}

fn has_window_decoration_from_xprop(output: &str) -> bool {
    let regex = Regex::new(r"_MOTIF_WM_HINTS\(CARDINAL\) = \d+, \d+, (\d+), \d+, \d+").unwrap();
    for line in output.lines() {
        match regex.captures(&line) {
            Some(c) => match c.get(1).unwrap().as_str() {
                "0" => return false,
                _ => return true
            },
            None => continue
        }
    }

    true
}

fn parse_window_decoration_from_xprop(output: &str) -> Option<WindowDecoration> {
    let regex = Regex::new(r"_NET_FRAME_EXTENTS\(CARDINAL\) = (\d+), (\d+), (\d+), (\d+)").unwrap();
    for line in output.lines() {
        match regex.captures(&line) {
            Some(c) => return Some(WindowDecoration {
                l: c.get(1).unwrap().as_str().parse().unwrap(),
                r: c.get(2).unwrap().as_str().parse().unwrap(),
                t: c.get(3).unwrap().as_str().parse().unwrap(),
                b: c.get(4).unwrap().as_str().parse().unwrap()
            }),
            None => continue
        }
    }

    None
}

fn get_window_decoration(id: WindowID, xprop: &str) -> Option<WindowDecoration> {
    let output = run_with_check(&[xprop, "-id", &id.to_string()]).stdout;
    let output = String::from_utf8(output).unwrap();
    let decorated = has_window_decoration_from_xprop(&output);

    if decorated {
        parse_window_decoration_from_xprop(&output)
    } else {
        None
    }
}

fn move_window(id: WindowID, x: i32, y: i32, xdotool: &str) {
    run_with_check(&[xdotool,
                     "windowmove",
                     &id.to_string(),
                     &x.to_string(),
                     &y.to_string()]);
}

fn move_window_in_grid(id: WindowID,
                       nrows: i32, ncolumns: i32,
                       row: i32, column: i32,
                       decoration: &Option<WindowDecoration>,
                       xdotool: &str, xwininfo: &str) {
    let display = get_display_geometry(xdotool);
    let window = get_window_geometry(id, xwininfo);

    assert!(nrows > 0);
    assert!(ncolumns > 0);
    assert!(row < nrows);
    assert!(column < ncolumns);

    let cell_w = display.w / ncolumns;
    let cell_h = display.h / nrows;
    let (win_w, win_h) = if let Some(deco) = decoration {
        (window.w + deco.l + deco.r,
         window.h + deco.t + deco.b)
    } else {
        (window.w, window.h)
    };

    let x = cell_w * column + (cell_w - win_w) / 2;
    let y = cell_h * row + (cell_h - win_h) / 2;

    let x = cmp::max(cmp::min(x, display.w - win_w), 0);
    let y = cmp::max(cmp::min(y, display.h - win_h), 0);

    move_window(id, x, y, xdotool);
}

fn resize_window(id: WindowID, width: i32, height: i32, xdotool: &str) {
    run_with_check(&[xdotool,
                     "windowsize",
                     &id.to_string(),
                     &width.to_string(),
                     &height.to_string()]);
}

fn resize_window_to_fill(id: WindowID, nrows: i32, ncolumns: i32,
                         gap_w: i32, gap_h: i32,
                         decoration: &Option<WindowDecoration>,
                         display: &DisplayGeometry, xdotool: &str) {
    let mut width = display.w / ncolumns - gap_w * 2;
    let mut height = display.h / nrows - gap_h * 2;

    if let Some(deco) = decoration {
        width = width - deco.l - deco.r;
        height = height - deco.t - deco.b;
    }

    resize_window(id, width, height, xdotool);
}


const USAGE: &'static str = "
Gridded Window Placer

Usage:
    gwip move --grid=<NROWSxNCOLUMNS> --place=<ROW,COLUMN> [options]
    gwip -h | --help

Options:
    -h, --help               Show this screen.
    --grid=<NROWSxNCOLUMNS>  How to divide screen.
                             Example: \"--grid=2x1\"
    --place=<ROW,COLUMN>     Where move the focused window to.
                             Example: \"--place=0,0\"
    --fill                   Resize the window to fill the cell where the
                             the window will be moved to.
    --gap=<WIDTHxHEIGHT>     For \"--fill\", do not fill each edge of the
                             window.  WIDTH is a width of left and right edge.
                             HEIGHT is a height of top and bottom edge.
                             WIDTH and HEIGHT is a number or a percentage.
                             A number means the number of pixels, a percentage
                             means the percentage of the desktop width
                             or height.
    --ignore-decoration      Ignore window decoration for calcurating
                             the window size and the window position.
                             If this opton is set, xprop is not necessary
                             to run gwip.
    --xdotool=<cmd>          Command of xdotool. [default: xdotool]
    --xwininfo=<cmd>         Command of xwininfo. [default: xwininfo]
    --xprop=<cmd>            Command of xprop. [default: xprop]

Commands:
    move
        Move the focused window to the specified place.
        The display is divided into grid by \"--grid\" parameter.
        The window will be moved to the center of the cell in the grid
        specified by \"--place\" parameter.
        The row and the column of a cell is counted from top left
        and zero based.

        Example:
            $ gwip move --grid=2x3 --place=0,1

                            ncolumns = 3
                      |-----------------------|

                   -  +-------+-------+-------+
                   |  |  0,0  |  0,1  |  0,2  |
                   |  |       | here! |       |
         nrows = 2 |  |-------+-------+-------|
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
    flag_ignore_decoration: bool,
    flag_xdotool: String,
    flag_xwininfo: String,
    flag_xprop: String,
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
        let nrows: i32 = parse_grid(grid[0]);
        let ncolumns: i32 = parse_grid(grid[1]);


        fn parse_place(s: &str) -> i32 {
            let msg = &format!("Invalid argument, \"{}\" in --place", s);
            s.parse().expect(msg)
        }

        let place: Vec<&str> = args.flag_place.split(",").collect();
        assert!(place.len() == 2);
        let row: i32 = parse_place(place[0]);
        let column: i32 = parse_place(place[1]);

        let id = get_focused_window_id(&args.flag_xdotool);
        let decoration = if args.flag_ignore_decoration {
            None
        } else {
            get_window_decoration(id, &args.flag_xprop)
        };

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

            resize_window_to_fill(id, nrows, ncolumns,
                                  gap_w, gap_h,
                                  &decoration, &display,
                                  &args.flag_xdotool);
        }
        move_window_in_grid(id, nrows, ncolumns, row, column, &decoration,
                            &args.flag_xdotool, &args.flag_xwininfo);
    }
}
