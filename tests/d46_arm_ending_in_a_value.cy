// `instead` replaced the bare last line, which read like a leftover.
// trace: D139, D45
fn first_word [] -> str:
    fails [too_long];

    return "word";
end fn first_word

fn.main [] -> void:
    let $w: str = first_word() otherwise:
        case too_long:
            "…";
    end otherwise
    return void;
end fn.main
