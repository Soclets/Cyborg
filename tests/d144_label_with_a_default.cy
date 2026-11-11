// trace: D144, D114
import std;
import os;

fam.unit screen:
    screen.pixels: base;
end fam.unit screen

fn draw [$x: i32[unit: screen.pixels]] returns void:
    labels [$x as at.screen.pixels];
    defaults [$x set_to 3];
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();

    std::io::fmt_print(Lend.mut $out, "{}{newln}", ($x as i32));
    return void;
end fn draw

fn.main [none] returns void:
    draw();
    draw(at.screen.pixels: 4);
    return void;
end fn.main
