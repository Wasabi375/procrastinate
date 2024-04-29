use nom::{combinator::eof, error::ParseError, sequence::pair, IResult, InputLength, Parser};

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

pub fn consume_all<I, O, E, P>(mut parser: P) -> impl FnMut(I) -> IResult<I, O, E>
where
    P: Parser<I, O, E>,
    I: InputLength + Clone,
    E: ParseError<I>,
{
    move |input: I| {
        let (input, (o, _)) = pair(|input| parser.parse(input), eof)(input.clone())?;
        Ok((input, o))
    }
}
