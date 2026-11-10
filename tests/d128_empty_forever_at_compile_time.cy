// An endless loop in compile-time code reaches the step limit.
// trace: D128, D89

fn.pure.baked spin [none] -> i32:
    forever:
    end forever
end fn.pure.baked spin

const $a: i32 = spin();

fn.main [none] -> void:
    return void;
end fn.main
