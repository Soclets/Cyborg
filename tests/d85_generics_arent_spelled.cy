// Generics are spelled now, so this older spelling, with a space before `<` and `?T`, is refused.
// trace: D85, D234
@config(default);

fn largest <?T: ordered> [$a: ?T, $b: ?T] -> ?T:
    return $a;
end fn largest

fn.main [] -> void:
    return void;
end fn.main
