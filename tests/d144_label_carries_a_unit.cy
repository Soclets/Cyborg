// trace: D144
import std;
import os;

fam.unit screen:
    screen.pixels: base;
end fam.unit screen

fn draw [$x: i32[unit: screen.pixels]] returns void:
    labels [$x as at.screen.pixels];
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();

    std::io::fmt_print(Lend.mut $out, "{}{newln}", ($x as i32));
    return void;
end fn draw

fn.main [none] returns void:
    draw(at.screen.pixels: 5);
    draw(at.screen.pixels: 6 screen.pixels);
    let $n: i32 = 7;
    draw(at.screen.pixels: $n);
    let $p: i32[unit: screen.pixels] = 8 screen.pixels;
    draw(at.screen.pixels: $p);
    return void;
end fn.main
