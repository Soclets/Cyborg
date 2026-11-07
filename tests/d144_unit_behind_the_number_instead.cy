// The writer picks this or a unit behind the number, and here a plain number has no label's unit.
// trace: D144, D101
fam.unit screen:
    screen.pixels: base;
end fam.unit screen

fn draw [$x: i32.screen.pixels] -> void:
    labels [$x as at];

    return void;
end fn draw

fn.main [] -> void:
    draw(at: 5 screen.pixels);
    draw(at: 5);
    return void;
end fn.main
