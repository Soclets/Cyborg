// Compile-time code always finishes, through a step limit that refuses it.
// trace: D89, D91

fn.pure.baked spin [$n: i32] -> i32:
    return spin($n + 1);
end fn.pure.baked spin

const $a: i32 = spin(0);

fn.main [none] -> void:
    return void;
end fn.main
