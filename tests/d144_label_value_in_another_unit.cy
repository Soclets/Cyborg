// trace: D144
@config(default);

fam.unit screen:
    screen.pixels: base;
    screen.points: base;
end fam.unit screen

fn draw [$x: i32.screen.pixels] -> void:
    labels [$x as at.screen.pixels];

    return void;
end fn draw

fn.main [] -> void:
    draw(at.screen.pixels: 5 screen.points);
    return void;
end fn.main
