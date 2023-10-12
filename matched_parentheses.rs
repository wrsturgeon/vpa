fn main() {
    let matched_parentheses = fixpoint("S")
        >> (empty()
            | (open(Paren, '(') >> recurse("S") >> close(Paren, ')'))
            | (recurse("S") >> recurse("S")));
}
