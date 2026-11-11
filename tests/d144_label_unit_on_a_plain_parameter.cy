// trace: D144

fam.unit screen:
    screen.pixels: base;
end fam.unit screen

fn draw [$x: i32] returns void:
    labels [$x as at.screen.pixels];

    return void;
end fn draw

fn.main [none] returns void:
    return void;
end fn.main
