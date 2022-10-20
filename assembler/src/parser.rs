use crate::Binary;
use crate::Instruction;
use crate::Operand;
use crate::Operation;
use crate::SectionType;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::bytes::complete::take_while;
use nom::character::is_alphabetic;
use nom::combinator::map;
use nom::combinator::map_opt;
use nom::multi::separated_list0;
use nom::number::complete::double;
use nom::sequence::preceded;
use nom::sequence::terminated;
use nom::sequence::tuple;
use nom::IResult;

fn sp(i: &str) -> IResult<&str, &str> {
    let chars = " \t\r\n";

    take_while(move |c| chars.contains(c) || chars.len() == 0)(i)
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

struct Op3<'a> {
    rs: Operand<'a>,
    rt: Operand<'a>,
    rd: Operand<'a>,
}

struct Op2Im<'a> {
    rs: Operand<'a>,
    rt: Operand<'a>,
    im: Operand<'a>,
}

fn op2im(i: &str) -> IResult<&str, Op2Im> {
    let (i, rt) = operand(i)?;
    let (i, rs) = c_operand(i)?;
    let (i, im) = preceded(comma, operand)(i)?;

    return Ok((i, Op2Im { rs, rt, im }));
}

fn op3(i: &str) -> IResult<&str, Op3> {
    map(tuple((operand, c_operand, c_operand)), |(rd, rs, rt)| Op3 {
        rs,
        rt,
        rd,
    })(i)
}

fn branch_instruction(i: &str) -> IResult<&str, Instruction> {
    let beq = map(tuple((tag("beq"), op2im)), |(_, op2im)| {
        Instruction::ii(Operation(0x4), op2im.rs, op2im.rt, op2im.im)
    });
    let bne = map(tuple((tag("bne"), op2im)), |(_, op2im)| {
        Instruction::ii(Operation(0x5), op2im.rs, op2im.rt, op2im.im)
    });
    let slt = map(tuple((tag("slt"), op3)), |(_, op3)| {
        Instruction::ri(
            Operation(0x0),
            op3.rs,
            op3.rt,
            op3.rd,
            Operand::Constant(0x0),
            Operand::Constant(0x2a),
        )
    });

    alt((beq, bne, slt))(i)
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

    let rt = operand;
    let im = preceded(comma, operand);
    let rs = preceded(char('('), terminated(operand, char(')')));

    map(tuple((alt((lw, sw)), rt, im, rs)), |(op, rt, im, rs)| {
        Instruction::ii(op, rs, rt, im)
    })(i)
}

fn arithmetic_with_immediate(i: &str) -> IResult<&str, Instruction> {
    let addi = map(tag("addi"), |_| Operation(0x8));
    let addiu = map(tag("addiu"), |_| Operation(0x9));
    let lui = map(tag("lui"), |_| Operation(0xf));

    map(tuple((alt((addiu, addi, lui)), op2im)), |(op, op2im)| {
        Instruction::ii(op, op2im.rs, op2im.rt, op2im.im)
    })(i)
}

fn arithmetic_with_register(i: &str) -> IResult<&str, Instruction> {
    let addu = map(tag("addu"), |_| 0x21);
    let subu = map(tag("subu"), |_| 0x23);
    let and = map(tag("and"), |_| 0x24);
    let or = map(tag("or"), |_| 0x25);

    map(tuple((alt((addu, subu, and, or)), op3)), |(fc, op3)| {
        Instruction::ri(
            Operation(0x0),
            op3.rs,
            op3.rt,
            op3.rd,
            Operand::Constant(0x0),
            Operand::Constant(fc),
        )
    })(i)
}

fn move_from(i: &str) -> IResult<&str, Instruction> {
    let mfhi = map(tag("mfhi"), |_| Operand::Constant(0x10));
    let mflo = map(tag("mflo"), |_| Operand::Constant(0x12));

    map(tuple((alt((mfhi, mflo)), operand)), |(fc, rd)| {
        Instruction::ri(
            Operation(0x0),
            Operand::Constant(0x0),
            Operand::Constant(0x0),
            rd,
            Operand::Constant(0x0),
            fc,
        )
    })(i)
}

fn shift_instruction(i: &str) -> IResult<&str, Instruction> {
    let sll = map(tag("sll"), |_| 0x0);
    let srl = map(tag("srl"), |_| 0x2);

    map(tuple((alt((sll, srl)), op2im)), |(fc, op2im)| {
        Instruction::ri(
            Operation(0x0),
            op2im.rs,
            op2im.rt,
            Operand::Constant(0x0),
            op2im.im,
            Operand::Constant(fc),
        )
    })(i)
}

fn arithmetic_with_hi_lo(i: &str) -> IResult<&str, Instruction> {
    let multu = map(tag("multu"), |_| 0x19);
    let mult = map(tag("mult"), |_| 0x18);
    let divu = map(tag("divu"), |_| 0x1b);
    let div = map(tag("div"), |_| 0x1a);

    map(
        tuple((alt((multu, mult, divu, div)), operand, c_operand)),
        |(fc, rs, rt)| {
            Instruction::ri(
                Operation(0x0),
                rs,
                rt,
                Operand::Constant(0x0),
                Operand::Constant(0x0),
                Operand::Constant(fc),
            )
        },
    )(i)
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

fn section(i: &str) -> IResult<&str, Instruction> {
    let data = map(tag("data"), |_| Instruction::Section(SectionType::Data));
    let text = map(tag("text"), |_| Instruction::Section(SectionType::Text));
    let globl = map(
        preceded(
            tuple((tag("globl"), sp)),
            separated_list0(
                preceded(sp, terminated(tag(","), sp)),
                map(string, |s| s.to_string()),
            ),
        ),
        |w| Instruction::Section(SectionType::Globl(w)),
    );

    let word = map(
        preceded(
            tuple((tag("word"), sp)),
            separated_list0(preceded(sp, terminated(tag(","), sp)), number),
        ),
        |w| Instruction::Section(SectionType::Word(w)),
    );

    let space = map(preceded(tuple((tag("space"), sp)), number), |n| {
        Instruction::Section(SectionType::Space(n))
    });
    preceded(tag("."), alt((data, word, space, text, globl)))(i)
}

fn comment(i: &str) -> IResult<&str, &str> {
    map(
        tuple((sp, tag("#"), take_while(|c| c != '\n'), sp)),
        |(_, _, c, _)| c,
    )(i)
}

pub fn one_parse(mut i: &str) -> IResult<&str, Instruction> {
    while let Ok((r, _)) = comment(i) {
        i = r;
    }
    preceded(
        sp,
        terminated(
            alt((
                section,
                syscall,
                def_label,
                branch_instruction,
                memory_instruction,
                jump_instruction,
                arithmetic_with_immediate,
                arithmetic_with_register,
                arithmetic_with_hi_lo,
                shift_instruction,
                move_from,
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

            return Err(format!("Line: {} {}", line_number - 0, &input[cursor..]));
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

    let input = ".data";
    assert_eq!(
        one_parse(input),
        Ok(("", Instruction::Section(SectionType::Data)))
    );

    let input = ".word 1, 2, 3";
    assert_eq!(
        one_parse(input),
        Ok(("", Instruction::Section(SectionType::Word(vec![1, 2, 3]))))
    );
    let input = ".space 20";
    assert_eq!(
        one_parse(input),
        Ok(("", Instruction::Section(SectionType::Space(20))))
    );
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
