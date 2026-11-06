// The shape of a small tokenizer step: a union of tokens, passed around and given away.
// trace: D2, D3, D9, D33, D57, D104
import std;

union.tagged token:
    number[value: i32]
    word[text: str]
    done
end union.tagged token

fn describe [$t: Given token] -> str:
    match $t:
        case token.number:
            return $t.value as str;
        case token.word:
            return Copy $t.text;
        case token.done:
            return "(done)";
    end match
end fn describe

fn make_word [$text: Given str] -> token:
    return token.word[text = Give $text];
end fn make_word

fn.main [] -> void:
    let $name: str = "hello";
    std::println(describe(token.number[value = 42]));
    std::println(describe(make_word(Give $name)));
    std::println(describe(token.done));

    let mut $state: token = token.number[value = 1];
    if $state: token.number:
        std::println($state.value as str);
        change $state = token.done;
    end if
    if $state: token.done:
        std::println("finished");
    end if
    return void;
end fn.main
