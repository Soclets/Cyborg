// Compile-time code always finishes, through a step limit that refuses it.
// trace: D89, D91

fn.pure.baked spin [$n: i32] returns i32:
    return spin($n + 1);
end fn.pure.baked spin

const $a: i32 set_to spin(0);

fn.main [none] returns void:
    return void;
end fn.main
