// trace: D144, D113
fam.unit screen:
    screen.pixels: base;
end fam.unit screen

fn draw [$x: i32.screen.pixels] -> void:
    labels [$x as at.screen.pixels];

    return void;
end fn draw

fn.main [] -> void:
    draw(5);
    return void;
end fn.main
