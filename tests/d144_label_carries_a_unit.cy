// trace: D144
import std;

fam.unit screen:
    screen.pixels: base;
end fam.unit screen

fn draw [$x: i32.screen.pixels] -> void:
    labels [$x as at.screen.pixels];

    std::println(($x as i32) as str);
    return void;
end fn draw

fn.main [] -> void:
    draw(at.screen.pixels: 5);
    draw(at.screen.pixels: 6 screen.pixels);
    let $n: i32 = 7;
    draw(at.screen.pixels: $n);
    let $p: i32.screen.pixels = 8 screen.pixels;
    draw(at.screen.pixels: $p);
    return void;
end fn.main
