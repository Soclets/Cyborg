// An endless loop in compile-time code reaches the step limit.
// trace: D128, D89

fn.pure.baked spin [none] returns i32:
    forever:
    end forever
end fn.pure.baked spin

const $a: i32 set_to spin();

fn.main [none] returns void:
    return void;
end fn.main
