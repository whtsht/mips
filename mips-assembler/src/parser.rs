use crate::Binary;
use crate::Instruction;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::bytes::complete::take_while;
use nom::character::is_alphabetic;
use nom::combinator::map;
use nom::combinator::map_opt;
use nom::number::complete::double;
use nom::sequence::preceded;
use nom::sequence::terminated;
use nom::sequence::tuple;
use nom::IResult;

fn sp(i: &str) -> IResult<&str, &str> {
    let chars = " \t\r\n";

    take_while(move |c| chars.contains(c))(i)
}

fn comma(i: &str) -> IResult<&str, ()> {
    Ok((tuple((sp, tag(","), sp))(i)?.0, ()))
}

fn string(i: &str) -> IResult<&str, &str> {
    take_while(move |c: char| is_alphabetic(c as u8) || ('0' <= c && c <= '9'))(i)
}

fn binary_from_number(input: &str) -> IResult<&str, Binary> {
    map(double, |n| n as Binary)(input)
}

fn binary_from_name(i: &str) -> IResult<&str, Binary> {
    map_opt(string, |i| match i {
        "zero" => Some(0),
        "at" => Some(1),
        "v0" => Some(2),
        "v1" => Some(3),
        "a0" => Some(4),
        "a1" => Some(5),
        "a2" => Some(6),
        "a3" => Some(7),
        "t0" => Some(8),
        "t1" => Some(9),
        "t2" => Some(10),
        "t3" => Some(11),
        "t4" => Some(12),
        "t5" => Some(13),
        "t6" => Some(14),
        "t7" => Some(15),
        "s0" => Some(16),
        "s1" => Some(17),
        "s2" => Some(18),
        "s3" => Some(19),
        "s4" => Some(20),
        "s5" => Some(21),
        "s6" => Some(22),
        "s7" => Some(23),
        "s8" => Some(24),
        "s9" => Some(25),
        "k0" => Some(26),
        "k1" => Some(27),
        "gp" => Some(28),
        "sp" => Some(29),
        "fp" => Some(30),
        "ra" => Some(31),
        _ => None,
    })(i)
}

fn r(i: &str) -> IResult<&str, Binary> {
    preceded(
        tuple((sp, tag("$"))),
        alt((binary_from_number, binary_from_name)),
    )(i)
}

fn rc(i: &str) -> IResult<&str, Binary> {
    preceded(
        comma,
        preceded(tag("$"), alt((binary_from_number, binary_from_name))),
    )(i)
}

fn r2im(i: &str) -> IResult<&str, (Binary, Binary, Binary)> {
    let (i, rs) = r(i)?;
    let (i, rt) = rc(i)?;
    let (i, im) = preceded(comma, binary_from_number)(i)?;

    return Ok((i, (rs, rt, im)));
}

fn r3(i: &str) -> IResult<&str, (Binary, Binary, Binary)> {
    tuple((r, rc, rc))(i)
}

fn jump_instruction(i: &str) -> IResult<&str, Instruction> {
    let (i, _) = tag("jr")(i)?;
    let (i, rs) = r(i)?;
    Ok((i, Instruction::ri(0x0, rs, 0x0, 0x0, 0x0, 0x8)))
}

fn memory_instruction(i: &str) -> IResult<&str, Instruction> {
    use nom::character::complete::char;
    let lw = map(tag("lw"), |_| 0x23);
    let sw = map(tag("sw"), |_| 0x2b);

    let (i, op) = alt((lw, sw))(i)?;
    let (i, rt) = r(i)?;
    let (i, im) = preceded(comma, binary_from_number)(i)?;
    let (i, rs) = preceded(char('('), terminated(r, char(')')))(i)?;

    Ok((i, Instruction::ii(op, rs, rt, im)))
}

fn arithmetic_with_immediate(i: &str) -> IResult<&str, Instruction> {
    let addi = map(tag("addi"), |_| 0x8);
    let addiu = map(tag("addiu"), |_| 0x9);

    let (i, op) = alt((addi, addiu))(i)?;
    let (i, (rt, rs, im)) = r2im(i)?;
    Ok((i, Instruction::ii(op, rs, rt, im)))
}

fn arithmetic_with_register(i: &str) -> IResult<&str, Instruction> {
    let addu = map(tag("addu"), |_| 0x21);
    let subu = map(tag("subu"), |_| 0x23);
    let and = map(tag("and"), |_| 0x24);
    let or = map(tag("or"), |_| 0x25);

    let (i, fc) = alt((addu, subu, and, or))(i)?;
    let (i, (rd, rs, rt)) = r3(i)?;
    Ok((i, Instruction::ri(0x0, rs, rt, rd, 0x0, fc)))
}

//fn label(i: &str) -> IResult<&str, &str> {}

pub fn one_parse(i: &str) -> IResult<&str, Instruction> {
    let syscall = map(tag("syscall"), |_| Instruction::ri(0, 0, 0, 0, 0, 0xc));

    preceded(
        sp,
        terminated(
            nom::branch::alt((
                syscall,
                memory_instruction,
                jump_instruction,
                arithmetic_with_immediate,
                arithmetic_with_register,
            )),
            sp,
        ),
    )(i)
}

pub fn parse(mut i: &str) -> Vec<Instruction> {
    let mut tokens = Vec::new();
    let syscall = map(tag("syscall"), |_| Instruction::ri(0, 0, 0, 0, 0, 0xc));

    let mut parser = preceded(
        sp,
        terminated(
            nom::branch::alt((
                syscall,
                memory_instruction,
                jump_instruction,
                arithmetic_with_immediate,
                arithmetic_with_register,
            )),
            sp,
        ),
    );

    while let Ok((rest, instr)) = parser(i) {
        tokens.push(instr);
        i = rest;
    }

    tokens
}

fn _p(i: &str) -> IResult<&str, &str> {
    tag("abc")(i)
}
///1001010

#[test]
fn test_parse() {
    let input = "addi $1, $2, -10";
    assert_eq!(
        one_parse(input),
        Ok(("", Instruction::ii(0x8, 0x2, 0x1, (u32::MAX - 9) as i32)))
    );

    let input = "addu $3, $5, $2";
    assert_eq!(
        one_parse(input),
        Ok(("", Instruction::ri(0x0, 0x5, 0x2, 0x3, 0x0, 0x21)))
    );

    let input = "or $10, $11, $12";
    assert_eq!(
        one_parse(input),
        Ok(("", Instruction::ri(0x0, 0xb, 0xc, 0xa, 0x0, 0x25)))
    );

    let input = "and $t0, $t1, $t2";
    assert_eq!(
        one_parse(input),
        Ok(("", Instruction::ri(0x0, 0x9, 0xa, 0x8, 0x0, 0x24)))
    );

    let input = "lw $t0, 400($t1)";
    assert_eq!(
        one_parse(input),
        Ok(("", Instruction::ii(0x23, 0x9, 0x8, 400)))
    );
}
