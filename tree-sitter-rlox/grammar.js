/**
 * @file A tree-sitter parser for the language rlox
 * @author Abreham Kassa <abrehamkassa19@gmail.com>
 * @license MIT
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

module.exports = grammar({
  name: "rlox",
  rules: {
    source_file: $ => repeat($._statement),
    _statement: $ => choice($.var_statement, $.function_definition, $.print_statement, $.expr_statement, $.return_statement, $.if_statement, $.while_statement, $.for_statement),
    if_statement: $ => seq("if", "(", $._expression, ")", choice($.block, $._statement)),
    while_statement: $ => seq("while", "(", $._expression, ")", $.block),
    for_statement: $ => seq("for", "(", optional(choice($.var_statement, $._expression)), ";", optional($._expression), ";", optional($._expression), ")", $.block),
    return_statement: $ => seq("return", optional($._expression), ';'),
    expr_statement: $ => seq($._expression, ";"),
    print_statement: $ => seq("print", $._expression, ';'),
    var_statement: $ => seq("var", field("name", $.identifier), optional($.initializer), ";"),
    parameter_list: $ => seq("(", optional(seq(field("name", $.identifier), repeat(seq(/,\s*/, field("name", $.identifier))))), ")"),
    block: $ => seq("{", repeat($._statement), "}"),
    function_definition: $ => seq("fun", field("name", $.identifier), field("parameters", $.parameter_list), field("body", $.block)),
    initializer: $ => seq("=", $._expression),

    primary: $ => prec(9, choice($.identifier, $.number, $.string, "true", "false", "nil", seq("(", $._expression, ")"))),
    call_expression: $ => prec(8, choice($.primary, seq($.primary, "(", field("parameter", $._expression), optional(seq(',', field("parameter", $._expression))), ")" ))),
    unary_expression: $ => prec.right(7, seq(repeat1(choice("!", "-")), $._expression)),
    binary_expression: $ => choice(
      prec.left(6, seq($._expression, choice("*", "/"), $._expression)),
      prec.left(5, seq($._expression, choice("+", "-"), $._expression)),
      prec.left(4, seq($._expression, choice(">", "<", ">=", "<=", "=="), $._expression)),
      prec.left(3, seq($._expression, "and", $._expression)),
      prec.left(2, seq($._expression, "or", $._expression)),
      prec.right(1, seq($._expression, "=", $._expression))
    ),

    _expression: $ => choice($.unary_expression, $.binary_expression, $.call_expression, $.primary),
    identifier: $ => /[a-zA-Z_][a-zA-Z0-9_]*/,
    string: $ => /"([^"\\]|\\.)*"/,
    number: $ => /\d+/
  }
});
