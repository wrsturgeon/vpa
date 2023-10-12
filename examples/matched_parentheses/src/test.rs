use crate::{accept, parser, Run};
use quickcheck::*;

#[derive(Clone, Copy, Debug)]
enum Paren {
    Left,
    Right,
}

impl Arbitrary for Paren {
    #[inline]
    fn arbitrary(g: &mut Gen) -> Self {
        if bool::arbitrary(g) {
            Paren::Left
        } else {
            Paren::Right
        }
    }
    // shrinking doesn't really matter
}

impl From<Paren> for char {
    #[inline(always)]
    fn from(value: Paren) -> Self {
        match value {
            Paren::Left => '(',
            Paren::Right => ')',
        }
    }
}

quickcheck! {
    fn exactly_correct(v: Vec<Paren>) -> bool {
        let parser = parser();
        let chars = v.into_iter().map(char::from);
        let mut run = chars.clone().run(&parser);
        while run.next().is_some() {}
        run.ctrl == Ok(Err(accept(chars)))
    }
}
