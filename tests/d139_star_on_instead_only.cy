// trace: D139, D46
@config(default);

fam.handler word_failed:
    word_failed.use_placeholder:
        case too_long:
            instead "…";
    end word_failed.use_placeholder
end fam.handler word_failed

fn first_word [] -> str:
    fails [too_long];

    return "word";
end fn first_word

fn.main [] -> void:
    let $a: str = first_word() otherwise word_failed.*use_placeholder;
    return void;
end fn.main
