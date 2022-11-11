#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn no_operands() -> Result<(), (logging::Log, u32)> {
        let data = [
            ("illegal", vec![0b0100101011111100]),
            ("nop",     vec![0b0100111001110001]),
            ("reset",   vec![0b0100111001110000]),
            ("rte",     vec![0b0100111001110011]),
            ("rtr",     vec![0b0100111001110111]),
            ("rts",     vec![0b0100111001110101]),
            ("trapv",   vec![0b0100111001110110]),
        ];

        for (text, expected) in data {
            let mut asm = assembler::Assembler::default();
            assert_eq!(asm.run(text)?, &expected);
        }

        Ok(())
    }
}
