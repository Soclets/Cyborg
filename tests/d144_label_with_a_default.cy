// trace: D144, D114
import std;
@config(default);

fam.unit screen:
    screen.pixels: base;
end fam.unit screen

fn draw [$x: i32.screen.pixels] -> void:
    labels [$x as at.screen.pixels];
    defaults [$x = 3];

    std::io::fmt_print("{}{newln}", ($x as i32));
    return void;
end fn draw

fn.main [] -> void:
    draw();
    draw(at.screen.pixels: 4);
    return void;
end fn.main
