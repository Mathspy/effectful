use crate::ecma::{
    block, boolean, break_statement, declare, ident, if_statement, int, obj_pat, string,
    while_statement, Program,
};

pub fn machination() -> Program {
    Program {
        body: vec![
            declare::constant()
                .id(ident("main_call").into_pattern())
                .init(ident("main").call(vec![]).into_expression())
                .into_declaration()
                .or_statement(),
            while_statement(boolean(true).into_expression())
                .body(block(vec![
                    declare::constant()
                        .id(obj_pat(vec![("done", None), ("value", Some("eff"))]))
                        .init(
                            ident("main_call")
                                .member_access("next")
                                .call(vec![])
                                .into_expression(),
                        )
                        .into_declaration()
                        .or_statement(),
                    if_statement(ident("done").into_expression())
                        .body(block(vec![break_statement()
                            .into_statement()
                            .or_declaration()]))
                        .into_statement()
                        .or_declaration(),
                    if_statement(
                        ident("eff")
                            .member_access("ty")
                            .into_expression()
                            .strict_eq(string("__CONSOLE__").into_expression()),
                    )
                    .body(block(vec![ident("console")
                        .member_access("log")
                        .call(vec![ident("eff")
                            .member_access("args")
                            .dyn_member_access(int(0).into_expression())
                            .into_expression()])
                        .into_statement()
                        .or_declaration()]))
                    .into_statement()
                    .or_declaration(),
                ]))
                .into_statement()
                .or_declaration(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use crate::ecma::EcmaWriter;

    // TODO: This test should be redundant once we have a full generation test
    #[test]
    fn machination() {
        let mut output = Vec::new();
        let mut writer = EcmaWriter::new(&mut output);
        writer.write_program(&super::machination()).unwrap();
        let output = String::from_utf8(output).unwrap();

        assert_eq!(
            output,
            r#"const main_call=main();while(true){const {done,value:eff,}=main_call.next();if(done){break;}if(eff.ty==="__CONSOLE__"){console.log(eff.args[0],);}}"#
        );
    }
}
