use nom::{error::ParseError, Parser};

pub fn alt_many<I, O, E, P, Ps>(mut parsers: Ps) -> impl Parser<I, O, E>
where
    P: Parser<I, O, E>,
    I: Clone,
    for<'a> &'a mut Ps: IntoIterator<Item = &'a mut P>,
    E: ParseError<I>,
{
    move |input: I| {
        for parser in &mut parsers {
            if let r @ Ok(_) = parser.parse(input.clone()) {
                return r;
            }
        }
        nom::combinator::fail::<I, O, E>(input)
    }
}
