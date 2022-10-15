use crate::Binary;
use crate::Instruction;
use crate::Operand;
use crate::Operation;
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

fn number(input: &str) -> IResult<&str, Binary> {
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

pub fn label(i: &str) -> IResult<&str, Operand> {
    map(string, |s| Operand::Label(s))(i)
}

fn operand(i: &str) -> IResult<&str, Operand> {
    let rgt = map(preceded(tag("$"), alt((number, binary_from_name))), |b| {
        Operand::Register(b)
    });
    let constant = map(number, |n| Operand::Constant(n));
    preceded(sp, alt((rgt, constant, label)))(i)
}

fn c_operand(i: &str) -> IResult<&str, Operand> {
    preceded(comma, operand)(i)
}

fn op2im(i: &str) -> IResult<&str, (Operand, Operand, Operand)> {
    let (i, rs) = operand(i)?;
    let (i, rt) = c_operand(i)?;
    let (i, im) = preceded(comma, operand)(i)?;

    return Ok((i, (rs, rt, im)));
}

fn op3(i: &str) -> IResult<&str, (Operand, Operand, Operand)> {
    tuple((operand, c_operand, c_operand))(i)
}

fn branch_instruction(i: &str) -> IResult<&str, Instruction> {
    let beq = map(tuple((tag("beq"), op2im)), |(_, (rs, rt, im))| {
        Instruction::ii(Operation(0x4), rs, rt, im)
    });
    let bne = map(tuple((tag("bne"), op2im)), |(_, (rs, rt, im))| {
        Instruction::ii(Operation(0x5), rs, rt, im)
    });

    alt((beq, bne))(i)
}

fn jump_instruction(i: &str) -> IResult<&str, Instruction> {
    let j = map(tuple((tag("j"), preceded(sp, label))), |(_, ad)| {
        Instruction::ji(Operation(0x2), ad)
    });
    let jr = map(tuple((tag("jr"), operand)), |(_, rs)| {
        Instruction::ri(
            Operation(0x0),
            rs,
            Operand::Register(0x0),
            Operand::Register(0x0),
            Operand::Constant(0x0),
            Operand::Constant(0x8),
        )
    });
    alt((jr, j))(i)
}

fn memory_instruction(i: &str) -> IResult<&str, Instruction> {
    use nom::character::complete::char;
    let lw = map(tag("lw"), |_| Operation(0x23));
    let sw = map(tag("sw"), |_| Operation(0x2b));

    let (i, op) = alt((lw, sw))(i)?;
    let (i, rt) = operand(i)?;
    let (i, im) = preceded(comma, operand)(i)?;
    let (i, rs) = preceded(char('('), terminated(operand, char(')')))(i)?;

    Ok((i, Instruction::ii(op, rs, rt, im)))
}

fn arithmetic_with_immediate(i: &str) -> IResult<&str, Instruction> {
    let addi = map(tag("addi"), |_| Operation(0x8));
    let addiu = map(tag("addiu"), |_| Operation(0x9));

    let (i, op) = alt((addi, addiu))(i)?;
    let (i, (rt, rs, im)) = op2im(i)?;
    Ok((i, Instruction::ii(op, rs, rt, im)))
}

fn arithmetic_with_register(i: &str) -> IResult<&str, Instruction> {
    let addu = map(tag("addu"), |_| 0x21);
    let subu = map(tag("subu"), |_| 0x23);
    let and = map(tag("and"), |_| 0x24);
    let or = map(tag("or"), |_| 0x25);

    let (i, fc) = map(alt((addu, subu, and, or)), |n| Operand::Constant(n))(i)?;
    let (i, (rd, rs, rt)) = op3(i)?;
    Ok((
        i,
        Instruction::ri(Operation(0x0), rs, rt, rd, Operand::Constant(0x0), fc),
    ))
}

fn syscall(i: &str) -> IResult<&str, Instruction> {
    map(tag("syscall"), |_| {
        Instruction::ri(
            Operation(0),
            Operand::Register(0x0),
            Operand::Register(0x0),
            Operand::Register(0x0),
            Operand::Constant(0x0),
            Operand::Constant(0xc),
        )
    })(i)
}

fn def_label(i: &str) -> IResult<&str, Instruction> {
    map(terminated(string, tag(":")), |s| Instruction::LabelDef {
        name: s,
    })(i)
}

pub fn one_parse(i: &str) -> IResult<&str, Instruction> {
    preceded(
        sp,
        terminated(
            nom::branch::alt((
                syscall,
                def_label,
                branch_instruction,
                memory_instruction,
                jump_instruction,
                arithmetic_with_immediate,
                arithmetic_with_register,
            )),
            sp,
        ),
    )(i)
}

pub fn parse(input: &str) -> Result<Vec<Instruction>, String> {
    let mut tokens = Vec::new();

    let mut i = input;

    while i.len() > 0 {
        if let Ok((rest, instr)) = one_parse(i) {
            tokens.push(instr);
            i = rest;
        } else {
            let cursor = input.len() - i.len();
            let line_number = input[..cursor].chars().filter(|c| c == &'\n').count();

            return Err(format!("Line: {} {}", line_number - 1, &input[cursor..]));
        }
    }

    Ok(tokens)
}

#[test]
fn test_one_parse() {
    let input = "addi $1, $2, -10";
    assert_eq!(
        one_parse(input),
        Ok((
            "",
            Instruction::ii(
                Operation(0x8),
                Operand::Register(0x2),
                Operand::Register(0x1),
                Operand::Constant(-10)
            )
        ))
    );

    let input = "jr $ra";
    assert_eq!(
        one_parse(input),
        Ok((
            "",
            Instruction::ri(
                Operation(0x0),
                Operand::Register(31),
                Operand::Register(0x0),
                Operand::Register(0x0),
                Operand::Constant(0x0),
                Operand::Constant(0x8),
            )
        ))
    );

    // let input = "addu $3, $5, $2";
    // assert_eq!(
    //     one_parse(input),
    //     Ok(("", Token::ri(0x0, 0x5, 0x2, 0x3, 0x0, 0x21)))
    // );

    // let input = "or $10, $11, $12";
    // assert_eq!(
    //     one_parse(input),
    //     Ok(("", Token::ri(0x0, 0xb, 0xc, 0xa, 0x0, 0x25)))
    // );

    // let input = "and $t0, $t1, $t2";
    // assert_eq!(
    //     one_parse(input),
    //     Ok(("", Token::ri(0x0, 0x9, 0xa, 0x8, 0x0, 0x24)))
    // );

    // let input = "lw $t0, 400($t1)";
    // assert_eq!(one_parse(input), Ok(("", Token::ii(0x23, 0x9, 0x8, 400))));
}

#[test]
fn test_parse() {
    let input = r#"j L

addi $a0, $0, 34

L:
addi $a0, $0, -34

addi $v0, $0, 1
syscall
jr $ra
"#;

    let o = parse(input).unwrap();
    assert_eq!(o.len(), 7);
    assert_eq!(o[2], Instruction::LabelDef { name: "L" });
}
