// An arm ending in a value is shown from Radium, and isn't decided.
// trace: D45
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
