/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

const PREC = {
  field: 10,          // ., [], .+, +[, |<
  call: 9,
  unary: 8,          // !, ||, &&, include
  multiplicative: 7, // *, /, %
  additive: 6,       // +, -
};

module.exports = grammar({
  name: "hashbuild",

  extras: $ => [
    $.comment,
    /\s/,
  ],

  rules: {
    source_file: $ => $.expression,
    comment: $ => /#[^\n]*/,
    identifier: $ => /[a-zA-Z_][a-zA-Z_0-9]*/,
    bool: $ => choice("false", "true"),
    integer: $ => /[0-9]+/,
    float: $ => {
      const decimal_digit_seq = /[0-9]+/;
      const exponent = seq(/[eE]/, optional(/[-+]/), decimal_digit_seq);

      return token(prec(1, choice(
        seq(optional(/[-+]/), optional(decimal_digit_seq), '.', decimal_digit_seq, optional(exponent)),
        seq(optional(/[-+]/), decimal_digit_seq, '.', optional(exponent)),
        seq(optional(/[-+]/), decimal_digit_seq, exponent),
        seq(optional(/[-+]/), /inf(inity)?/),
        /NaN/
      )));
    },
    path: $ => token(seq(
      choice('/', './', '../'),
      /[a-zA-Z0-9_\-\.\/]+/
    )),
    string: $ => seq(
      '"',
      repeat(choice(
        $._string_content,
        $.escape_sequence
      )),
      '"'
    ),
    _string_content: $ => token(prec(1, /[^\n"\\]+/)),
    escape_sequence: $ => token(seq(
      '\\',
      choice(
        /[0abfnrtv\\'"]/,
        /x[0-9a-fA-F]{2}/,
        /u[0-9a-fA-F]{4}/,
        /u\{[0-9a-fA-F]{1,6}\}/
      )
    )),
    structure: $ => seq(
      '{',
      repeat(seq($.structure_field, ",")),
      optional($.structure_field),
      '}'
    ),
    structure_field: $ => seq(
      $.expression,
      optional(seq(
        "=",
        $.expression
      )),
    ),
    function: $ => seq(
      "|",
      $.identifier,
      "|",
      $.expression,
    ),
    add: $ => prec.left(PREC.additive, seq(
      $.expression,
      "+",
      $.expression,
    )),
    subtract: $ => prec.left(PREC.additive, seq(
      $.expression,
      "-",
      $.expression,
    )),
    multiply: $ => prec.left(PREC.multiplicative, seq(
      $.expression,
      "*",
      $.expression,
    )),
    divide: $ => prec.left(PREC.multiplicative, seq(
      $.expression,
      "/",
      $.expression,
    )),
    modulo: $ => prec.left(PREC.multiplicative, seq(
      $.expression,
      "%",
      $.expression,
    )),
    not: $ => prec(PREC.unary, seq(
      "!",
      $.expression,
    )),
    or: $ => prec(PREC.unary, seq(
      "||",
      $.expression,
    )),
    and: $ => prec(PREC.unary, seq(
      "&&",
      $.expression,
    )),
    field_get: $ => prec.left(PREC.field, seq(
      $.expression,
      ".",
      $.identifier,
    )),
    get: $ => prec.left(PREC.field, seq(
      $.expression,
      "[",
      $.expression,
      "]",
    )),
    field_set: $ => prec.left(PREC.field, seq(
      $.expression,
      ".+",
      $.identifier,
      $.expression,
    )),
    set: $ => prec.left(PREC.field, seq(
      $.expression,
      "+[",
      $.expression,
      "]",
      $.expression,
    )),
    set_all: $ => prec.left(PREC.field, seq(
      $.expression,
      "|<",
      $.expression,
    )),
    call: $ => prec.left(PREC.call, seq(
      $.expression,
      $.expression,
    )),
    include: $ => prec(PREC.unary, seq(
      "include",
      $.expression,
    )),
    expression: $ => choice(
      $.identifier,
      $.bool,
      $.integer,
      $.float,
      $.path,
      $.string,
      $.structure,
      $.function,
      $.add,
      $.subtract,
      $.multiply,
      $.divide,
      $.modulo,
      $.not,
      $.or,
      $.and,
      $.field_get,
      $.get,
      $.field_set,
      $.set,
      $.set_all,
      $.call,
      $.include,
    ),
  }
});
