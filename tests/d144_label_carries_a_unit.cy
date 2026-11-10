// trace: D144
@config(default);
import std;

fam.unit screen:
    screen.pixels: base;
end fam.unit screen

fn draw [$x: i32[unit: screen.pixels]] -> void:
    labels [$x as at.screen.pixels];

    std::io::fmt_print("{}{newln}", ($x as i32));
    return void;
end fn draw

fn.main [] -> void:
    draw(at.screen.pixels: 5);
    draw(at.screen.pixels: 6 screen.pixels);
    let $n: i32 = 7;
    draw(at.screen.pixels: $n);
    let $p: i32[unit: screen.pixels] = 8 screen.pixels;
    draw(at.screen.pixels: $p);
    return void;
end fn.main
